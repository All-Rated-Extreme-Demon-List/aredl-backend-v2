use crate::app_data::db::DbAppState;
use crate::auth::oauth::{OAuthClientConfig, OAuthProviderState};
use crate::error_handler::ApiError;
use crate::external_connections::{OAuthProvider, OAuthToken};
use crate::get_secret;
use crate::schema::oauth_tokens;

use base64::{
    engine::general_purpose::STANDARD, engine::general_purpose::URL_SAFE_NO_PAD, Engine as _,
};
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, OsRng, Payload},
    XChaCha20Poly1305, XNonce,
};
use chrono::{Duration as ChronoDuration, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::Deserialize;
use std::sync::OnceLock;
use tokio::sync::Mutex;

const ENCRYPTED_TOKEN_PREFIX: &str = "enc:v1:";

static TOKEN_CIPHER: OnceLock<Result<XChaCha20Poly1305, String>> = OnceLock::new();

#[derive(Debug, Clone)]
pub enum BackendGrantType {
    RefreshToken,
    ClientCredentials,
}

impl BackendGrantType {
    fn as_form_value(&self) -> &'static str {
        match self {
            Self::RefreshToken => "refresh_token",
            Self::ClientCredentials => "client_credentials",
        }
    }
}

pub struct BackendTokenState {
    pub token: Mutex<Option<OAuthToken>>,
    pub grant_type: BackendGrantType,
}

impl BackendTokenState {
    pub fn new(grant_type: BackendGrantType) -> Self {
        Self {
            token: Mutex::new(None),
            grant_type,
        }
    }
}

pub struct OAuthProviderContext {
    pub provider: OAuthProvider,
    pub api_base_uri: String,
    pub config: OAuthClientConfig,
    pub user_oauth: Option<OAuthProviderState>,
    pub backend_token: Option<BackendTokenState>,
}

#[derive(Deserialize)]
struct OAuthGrantResponse {
    access_token: String,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

impl OAuthProviderContext {
    pub fn new(
        provider: OAuthProvider,
        config: OAuthClientConfig,
        default_api_base_uri: String,
        backend_token: Option<BackendTokenState>,
    ) -> Result<Self, ApiError> {
        Ok(Self {
            provider,
            config: config.clone(),
            api_base_uri: config.api_base_uri.clone().unwrap_or(default_api_base_uri),
            user_oauth: Some(OAuthProviderState::new(config)?),
            backend_token,
        })
    }

    pub fn new_backend_only(
        provider: OAuthProvider,
        config: OAuthClientConfig,
        default_api_base_uri: String,
        backend_token: BackendTokenState,
    ) -> Result<Self, ApiError> {
        Ok(Self {
            provider,
            config: config.clone(),
            api_base_uri: config.api_base_uri.clone().unwrap_or(default_api_base_uri),
            user_oauth: None,
            backend_token: Some(backend_token),
        })
    }

    pub fn backend_token_state(&self) -> Result<&BackendTokenState, ApiError> {
        self.backend_token.as_ref().ok_or_else(|| {
            ApiError::new(
                500,
                &format!(
                    "Backend token state not initialized for provider {:?}",
                    self.provider
                ),
            )
        })
    }

    pub fn user_oauth(&self) -> Result<&OAuthProviderState, ApiError> {
        self.user_oauth.as_ref().ok_or_else(|| {
            ApiError::new(
                500,
                &format!(
                    "User OAuth not initialized for provider {:?}",
                    self.provider
                ),
            )
        })
    }

    async fn request_token(
        &self,
        grant_request: &[(&'static str, String)],
    ) -> Result<OAuthGrantResponse, ApiError> {
        let client = reqwest::Client::new();

        let resp = client
            .post(&self.config.token_uri)
            .form(grant_request)
            .send()
            .await
            .map_err(|e| {
                ApiError::new(
                    502,
                    &format!("Failed to request {:?} token: {e}", self.provider),
                )
            })?;

        let status = resp.status();

        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();

            return Err(ApiError::new(
                502,
                &format!(
                    "Failed to request {:?} token ({status}): {body}",
                    self.provider
                ),
            ));
        }

        resp.json::<OAuthGrantResponse>().await.map_err(|e| {
            ApiError::new(
                500,
                &format!("Failed to parse {:?} token response: {e}", self.provider),
            )
        })
    }

    fn read_stored_token(&self, db: &DbAppState) -> Result<OAuthToken, ApiError> {
        let token = self.read_optional_stored_token(db)?;

        let Some(token) = token else {
            return Err(ApiError::new(
                500,
                &format!(
                    "Missing {:?} backend OAuth token. Seed oauth_tokens for provider {:?}",
                    self.provider, self.provider
                ),
            ));
        };

        Ok(token)
    }

    fn read_optional_stored_token(&self, db: &DbAppState) -> Result<Option<OAuthToken>, ApiError> {
        let mut conn = db.connection()?;

        let stored_token = oauth_tokens::table
            .filter(oauth_tokens::provider.eq(self.provider))
            .select(OAuthToken::as_select())
            .first::<OAuthToken>(&mut conn)
            .optional()?;

        stored_token.map(decrypt_oauth_token).transpose()
    }

    async fn fetch_and_cache_token(&self, db: &DbAppState) -> Result<String, ApiError> {
        let backend_token_state = self.backend_token_state()?;
        let stored_token = match backend_token_state.grant_type {
            BackendGrantType::RefreshToken => Some(self.read_stored_token(db)?),
            BackendGrantType::ClientCredentials => self.read_optional_stored_token(db)?,
        };

        let mut grant_request = vec![
            ("client_id", self.config.client_id.clone()),
            ("client_secret", self.config.client_secret.clone()),
            (
                "grant_type",
                backend_token_state.grant_type.as_form_value().to_string(),
            ),
        ];

        let current_refresh_token = stored_token.and_then(|token| token.refresh_token);

        if let Some(refresh_token) = &current_refresh_token {
            grant_request.push(("refresh_token", refresh_token.clone()));
        }

        let response = self.request_token(&grant_request).await?;

        let expires_in = response.expires_in.unwrap_or(3600);
        let expires_at = Utc::now() + ChronoDuration::seconds(expires_in as i64);

        let refresh_token = response.refresh_token.or(current_refresh_token);

        let encrypted_access_token = encrypt_db_token_value(
            &response.access_token,
            &oauth_token_aad(self.provider, "access_token"),
        )?;

        let encrypted_refresh_token = refresh_token
            .as_deref()
            .map(|token| {
                encrypt_db_token_value(token, &oauth_token_aad(self.provider, "refresh_token"))
            })
            .transpose()?;

        let mut conn = db.connection()?;

        let updated_token = diesel::insert_into(oauth_tokens::table)
            .values((
                oauth_tokens::provider.eq(self.provider),
                oauth_tokens::access_token.eq(Some(encrypted_access_token.clone())),
                oauth_tokens::refresh_token.eq(encrypted_refresh_token.clone()),
                oauth_tokens::expires_at.eq(Some(expires_at)),
                oauth_tokens::updated_at.eq(Utc::now()),
            ))
            .on_conflict(oauth_tokens::provider)
            .do_update()
            .set((
                oauth_tokens::access_token.eq(Some(encrypted_access_token)),
                oauth_tokens::refresh_token.eq(encrypted_refresh_token),
                oauth_tokens::expires_at.eq(Some(expires_at)),
                oauth_tokens::updated_at.eq(Utc::now()),
            ))
            .returning(OAuthToken::as_returning())
            .get_result::<OAuthToken>(&mut conn)?;

        *backend_token_state.token.lock().await = Some(decrypt_oauth_token(updated_token)?);

        Ok(response.access_token)
    }

    fn valid_cached_access_token(token: &OAuthToken) -> Option<String> {
        let access_token = token.access_token.as_ref()?;
        let expires_at = token.expires_at?;

        let cache_expires_at = expires_at - ChronoDuration::seconds(60);

        (cache_expires_at > Utc::now()).then(|| access_token.clone())
    }

    pub async fn get_access_token(&self, db: &DbAppState) -> Result<String, ApiError> {
        let backend_token_state = self.backend_token_state()?;
        {
            let mutex = backend_token_state.token.lock().await;

            if let Some(cached_token) = &*mutex {
                if let Some(access_token) = Self::valid_cached_access_token(cached_token) {
                    return Ok(access_token);
                }
            }
        }

        let token = match backend_token_state.grant_type {
            BackendGrantType::RefreshToken => self.read_stored_token(db)?,
            BackendGrantType::ClientCredentials => match self.read_optional_stored_token(db)? {
                Some(token) => token,
                None => return self.fetch_and_cache_token(db).await,
            },
        };

        if let Some(access_token) = Self::valid_cached_access_token(&token) {
            *backend_token_state.token.lock().await = Some(token);
            return Ok(access_token);
        }

        self.fetch_and_cache_token(db).await
    }
}

fn decrypt_oauth_token(mut token: OAuthToken) -> Result<OAuthToken, ApiError> {
    let provider = token.provider;

    token.access_token = token
        .access_token
        .as_deref()
        .map(|value| decrypt_db_token_value(value, &oauth_token_aad(provider, "access_token")))
        .transpose()?;

    token.refresh_token = token
        .refresh_token
        .as_deref()
        .map(|value| decrypt_db_token_value(value, &oauth_token_aad(provider, "refresh_token")))
        .transpose()?;

    Ok(token)
}

pub fn oauth_token_aad(provider: OAuthProvider, field: &str) -> Vec<u8> {
    format!("oauth_tokens:v1:{provider:?}:{field}").into_bytes()
}

fn token_cipher() -> Result<&'static XChaCha20Poly1305, ApiError> {
    let cipher_result = TOKEN_CIPHER.get_or_init(|| {
        let secret = get_secret("OAUTH_TOKEN_ENCRYPTION_KEY");

        let key = STANDARD
            .decode(secret)
            .map_err(|_| "OAUTH_TOKEN_ENCRYPTION_KEY must be base64-encoded 32 random bytes")?;

        XChaCha20Poly1305::new_from_slice(&key)
            .map_err(|_| "OAUTH_TOKEN_ENCRYPTION_KEY must decode to exactly 32 bytes".to_string())
    });

    cipher_result.as_ref().map_err(|message| {
        ApiError::new(
            500,
            &format!("Failed to initialize backend OAuth token encryption: {message}"),
        )
    })
}

fn encrypt_db_token_value(value: &str, aad: &[u8]) -> Result<String, ApiError> {
    let cipher = token_cipher()?;
    let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);

    let ciphertext = cipher
        .encrypt(
            &nonce,
            Payload {
                msg: value.as_bytes(),
                aad,
            },
        )
        .map_err(|_| ApiError::new(500, "Failed to encrypt backend OAuth token"))?;

    Ok(format!(
        "{ENCRYPTED_TOKEN_PREFIX}{}:{}",
        URL_SAFE_NO_PAD.encode(nonce),
        URL_SAFE_NO_PAD.encode(ciphertext)
    ))
}

pub(crate) fn decrypt_db_token_value(value: &str, aad: &[u8]) -> Result<String, ApiError> {
    let Some(encrypted_value) = value.strip_prefix(ENCRYPTED_TOKEN_PREFIX) else {
        return Ok(value.to_string());
    };

    let (nonce_b64, ciphertext_b64) = encrypted_value
        .split_once(':')
        .ok_or_else(|| ApiError::new(500, "Invalid encrypted backend OAuth token format"))?;

    let nonce_bytes = URL_SAFE_NO_PAD
        .decode(nonce_b64)
        .map_err(|_| ApiError::new(500, "Invalid encrypted backend OAuth token nonce encoding"))?;

    let nonce_array: [u8; 24] = nonce_bytes
        .try_into()
        .map_err(|_| ApiError::new(500, "Invalid encrypted backend OAuth token nonce"))?;

    let ciphertext = URL_SAFE_NO_PAD
        .decode(ciphertext_b64)
        .map_err(|_| ApiError::new(500, "Invalid encrypted backend OAuth token encoding"))?;

    let nonce = XNonce::from(nonce_array);
    let cipher = token_cipher()?;

    let plaintext = cipher
        .decrypt(
            &nonce,
            Payload {
                msg: ciphertext.as_ref(),
                aad,
            },
        )
        .map_err(|_| ApiError::new(500, "Failed to decrypt backend OAuth token"))?;

    String::from_utf8(plaintext)
        .map_err(|_| ApiError::new(500, "Decrypted backend OAuth token is not UTF-8"))
}
