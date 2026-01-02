use crate::{error_handler::ApiError, get_optional_secret};
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Deserialize)]
struct GoogleOAuth2WebClientJson {
    web: GoogleOAuth2WebClient,
}

#[derive(Deserialize)]
struct GoogleOAuth2WebClient {
    client_id: String,
    client_secret: String,
    token_uri: String,
}

#[derive(Deserialize)]
struct GoogleOAuth2RefreshJson {
    refresh_token: String,
}

#[derive(Deserialize, Clone)]
pub struct GoogleOAuth2GrantResponse {
    pub access_token: String,
    pub expires_in: Option<u64>,
}

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

pub struct GoogleAuthState {
    grant_request: Vec<(&'static str, String)>,
    grant_uri: String,
    latest_token: Mutex<Option<CachedToken>>,
}

impl GoogleAuthState {
    async fn fetch_new_token(&self) -> Result<GoogleOAuth2GrantResponse, ApiError> {
        let client = reqwest::Client::new();
        let resp = client
            .post(&self.grant_uri)
            .form(&self.grant_request)
            .send()
            .await
            .map_err(|e| ApiError::new(502, &format!("Failed to request google token: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ApiError::new(
                502,
                &format!("Failed to request google token ({status}): {body}"),
            ));
        }

        let response: GoogleOAuth2GrantResponse = resp.json().await.map_err(|e| {
            ApiError::new(
                500,
                &format!("Failed to parse google refresh response: {e}"),
            )
        })?;

        Ok(response)
    }

    async fn fetch_and_cache_token(&self) -> Result<String, ApiError> {
        let response = self.fetch_new_token().await?;
        let expires_in = response.expires_in.unwrap_or(3600);
        let expires_at = Instant::now() + Duration::from_secs(expires_in.saturating_sub(60));

        let token = CachedToken {
            access_token: response.access_token.clone(),
            expires_at,
        };

        *self.latest_token.lock().await = Some(token);

        Ok(response.access_token)
    }

    pub async fn get_access_token(&self) -> Result<String, ApiError> {
        {
            let mutex = self.latest_token.lock().await;
            if let Some(cached_token) = &*mutex {
                if cached_token.expires_at > Instant::now() {
                    return Ok(cached_token.access_token.clone());
                }
            }
        }

        self.fetch_and_cache_token().await
    }

    pub async fn new() -> Option<Self> {
        let client_secret = get_optional_secret("GOOGLE_OAUTH_CLIENT")?;
        let refresh_secret = get_optional_secret("GOOGLE_OAUTH_REFRESH")?;

        let client: GoogleOAuth2WebClientJson = match serde_json::from_str(&client_secret) {
            Ok(c) => c,
            Err(e) => {
                tracing::warn!("Failed to parse GOOGLE_OAUTH_CLIENT: {}", e);
                return None;
            }
        };

        let refresh: GoogleOAuth2RefreshJson = match serde_json::from_str(&refresh_secret) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Failed to parse GOOGLE_OAUTH_REFRESH: {}", e);
                return None;
            }
        };

        let grant_request = vec![
            ("grant_type", "refresh_token".to_string()),
            ("refresh_token", refresh.refresh_token.clone()),
            ("client_id", client.web.client_id.clone()),
            ("client_secret", client.web.client_secret.clone()),
        ];

        Some(GoogleAuthState {
            grant_request,
            grant_uri: client.web.token_uri,
            latest_token: Mutex::new(None),
        })
    }
}
