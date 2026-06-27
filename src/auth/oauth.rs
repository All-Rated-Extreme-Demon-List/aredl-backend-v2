use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::{oauth_requests, oauth_tokens};
use crate::{get_optional_secret, get_secret};
use chrono::{DateTime, Utc};
use diesel::{
    Connection as _, ExpressionMethods as _, QueryDsl as _, RunQueryDsl as _, SelectableHelper as _,
};
use diesel_derive_enum::DbEnum;
use oauth2::basic::BasicClient;
use oauth2::{
    AuthType, AuthUrl, AuthorizationCode, ClientId, ClientSecret, CsrfToken, EndpointNotSet,
    EndpointSet, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse as _,
    TokenUrl,
};
use serde::{Deserialize, Serialize};
use strum_macros::Display;
use url::Url;
use utoipa::ToSchema;
use uuid::Uuid;

pub type OAuthClient =
    BasicClient<EndpointSet, EndpointNotSet, EndpointNotSet, EndpointNotSet, EndpointSet>;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, ToSchema, DbEnum, PartialEq, Eq)]
#[ExistingTypePath = "crate::schema::sql_types::OauthProvider"]
#[DbValueStyle = "PascalCase"]
pub enum OAuthProvider {
    Discord,
    Patreon,
    Google,
    Twitch,
}

#[derive(Debug, Clone, Queryable, Selectable, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = oauth_tokens)]
pub struct OAuthToken {
    pub provider: OAuthProvider,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Default, Serialize, Deserialize, ToSchema)]
pub struct OAuthOptions {
    pub callback: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OAuthCallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Deserialize, Clone, Display)]
#[serde(rename_all = "snake_case")]
pub enum OAuthAuthTypeConfig {
    BasicAuth,
    RequestBody,
}

impl From<OAuthAuthTypeConfig> for AuthType {
    fn from(value: OAuthAuthTypeConfig) -> Self {
        match value {
            OAuthAuthTypeConfig::BasicAuth => AuthType::BasicAuth,
            OAuthAuthTypeConfig::RequestBody => AuthType::RequestBody,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct OAuthClientConfig {
    pub client_id: String,
    pub client_secret: String,
    pub authorize_uri: Option<String>,
    pub token_uri: String,
    pub api_base_uri: Option<String>,
    pub redirect_uri: Option<String>,
    pub return_path: Option<String>,
    #[serde(default)]
    pub scopes: Vec<String>,
    pub use_pkce: Option<bool>,
    pub auth_type: Option<OAuthAuthTypeConfig>,
}

impl OAuthClientConfig {
    pub fn authorize_uri(&self) -> Result<String, ApiError> {
        self.authorize_uri.clone().ok_or_else(|| {
            ApiError::InternalServerError("OAuth client config missing authorize_uri")
        })
    }

    pub fn redirect_uri(&self) -> Result<String, ApiError> {
        match (
            get_optional_secret("OAUTH_RETURN_URI_BASE"),
            self.return_path.as_deref(),
        ) {
            (Some(base), Some(return_path)) => Ok(build_oauth_return_uri(&base, return_path)),
            _ => self.redirect_uri.clone().ok_or_else(|| {
                ApiError::InternalServerError(
                    "OAuth provider config missing return_path or redirect_uri",
                )
            }),
        }
    }

    pub fn auth_type(&self) -> OAuthAuthTypeConfig {
        self.auth_type
            .clone()
            .unwrap_or(OAuthAuthTypeConfig::RequestBody)
    }
}

pub(super) fn build_oauth_return_uri(base: &str, return_path: &str) -> String {
    let base = base.trim().trim_end_matches('/');
    let base = if base.starts_with("http://") || base.starts_with("https://") {
        base.to_owned()
    } else if base.starts_with("127.0.0.1") || base.starts_with("localhost") {
        format!("http://{base}")
    } else {
        format!("https://{base}")
    };

    let return_path = return_path.trim().trim_matches('/');
    format!("{base}/{return_path}")
}

#[derive(Clone)]
pub struct OAuthProviderState {
    pub client: OAuthClient,
    pub scopes: Vec<String>,
    pub use_pkce: bool,
}

impl OAuthProviderState {
    pub fn new(config: OAuthClientConfig) -> Result<Self, ApiError> {
        let authorize_uri = config.authorize_uri()?;
        let redirect_uri = config.redirect_uri()?;
        let client = BasicClient::new(ClientId::new(config.client_id))
            .set_client_secret(ClientSecret::new(config.client_secret))
            .set_auth_uri(AuthUrl::new(authorize_uri)?)
            .set_token_uri(TokenUrl::new(config.token_uri)?)
            .set_redirect_uri(RedirectUrl::new(redirect_uri)?)
            .set_auth_type(
                config
                    .auth_type
                    .unwrap_or(OAuthAuthTypeConfig::RequestBody)
                    .into(),
            );

        Ok(Self {
            client,
            scopes: config.scopes,
            use_pkce: config.use_pkce.unwrap_or(false),
        })
    }
}

impl OAuthOptions {
    pub fn validate(&self) -> Result<(), ApiError> {
        if let Some(callback) = &self.callback {
            let Ok(url) = Url::parse(callback) else {
                return Err(ApiError::BadRequest("Invalid callback URL"));
            };

            if !matches!(url.scheme(), "http" | "https") {
                return Err(ApiError::BadRequest("Invalid callback URL"));
            }

            let Some(host) = url.host_str().map(str::to_ascii_lowercase) else {
                return Err(ApiError::BadRequest("Invalid callback URL"));
            };

            let allow_localhost = get_optional_secret("AUTH_CALLBACK_ALLOW_LOCALHOST")
                .is_none_or(|value| value != "0" && !value.eq_ignore_ascii_case("false"));

            if allow_localhost && matches!(host.as_str(), "localhost" | "127.0.0.1" | "::1") {
                return Ok(());
            }

            let allowed_domains = get_secret("AUTH_CALLBACK_ALLOWED_DOMAINS")?;
            if allowed_domains
                .split(',')
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_ascii_lowercase)
                .any(|domain| host == *domain || host.ends_with(&format!(".{domain}")))
            {
                return Ok(());
            }

            return Err(ApiError::BadRequest("Invalid callback URL"));
        }

        Ok(())
    }
}

#[derive(Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name=oauth_requests)]
pub struct OAuthRequestData {
    pub csrf_state: String,
    pub pkce_verifier: Option<String>,
    pub callback: Option<String>,
    pub provider: OAuthProvider,
    pub user_id: Option<Uuid>,
}

impl OAuthRequestData {
    pub fn init_request(
        conn: &mut DbConnection,
        state: &OAuthProviderState,
        provider: OAuthProvider,
        callback: Option<String>,
        user_id: Option<Uuid>,
    ) -> Result<String, ApiError> {
        let mut authorization = state.client.authorize_url(CsrfToken::new_random);

        for scope in &state.scopes {
            authorization = authorization.add_scope(Scope::new(scope.clone()));
        }

        #[expect(
            clippy::if_then_some_else_none,
            reason = "using .then would move ownership"
        )]
        let pkce_verifier = if state.use_pkce {
            let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
            authorization = authorization.set_pkce_challenge(pkce_challenge);
            Some(pkce_verifier.secret().clone())
        } else {
            None
        };

        let (authorize_url, csrf_state) = authorization.url();

        diesel::insert_into(oauth_requests::table)
            .values((
                oauth_requests::csrf_state.eq(csrf_state.secret().clone()),
                oauth_requests::pkce_verifier.eq(pkce_verifier),
                oauth_requests::callback.eq(callback),
                oauth_requests::provider.eq(provider),
                oauth_requests::user_id.eq(user_id),
            ))
            .execute(conn)?;

        Ok(authorize_url.to_string())
    }

    pub fn consume_request(
        conn: &mut DbConnection,
        provider: OAuthProvider,
        csrf_state: &str,
    ) -> Result<Self, ApiError> {
        conn.transaction(|conn| {
            let request_data = oauth_requests::table
                .filter(oauth_requests::csrf_state.eq(csrf_state))
                .filter(oauth_requests::provider.eq(provider))
                .select(OAuthRequestData::as_select())
                .first::<OAuthRequestData>(conn)?;

            diesel::delete(
                oauth_requests::table
                    .filter(oauth_requests::csrf_state.eq(csrf_state))
                    .filter(oauth_requests::provider.eq(provider)),
            )
            .execute(conn)?;

            Ok(request_data)
        })
    }
}

pub async fn exchange_oauth_code(
    client: &OAuthClient,
    code: &str,
    pkce_verifier: Option<String>,
) -> Result<String, ApiError> {
    let mut request = client.exchange_code(AuthorizationCode::new(code.to_owned()));

    if let Some(pkce_verifier) = pkce_verifier {
        request = request.set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier));
    }

    let http_client = reqwest::Client::new();
    let token_response = request
        .request_async(&http_client)
        .await
        .map_err(|_err| ApiError::BadGateway("Failed to request token!"))?;

    Ok(token_response.access_token().secret().clone())
}
