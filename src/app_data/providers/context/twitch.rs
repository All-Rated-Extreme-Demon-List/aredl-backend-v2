use crate::{error_handler::ApiError, get_optional_secret};
use serde::Deserialize;
use std::time::{Duration, Instant};
use tokio::sync::Mutex;

#[derive(Deserialize, Clone)]
pub struct TwitchClientCredentialsGrantResponse {
    pub access_token: String,
    pub expires_in: u64,
}

struct CachedToken {
    access_token: String,
    expires_at: Instant,
}

pub struct TwitchAuthState {
    pub client_id: String,
    grant_request: Vec<(&'static str, String)>,
    grant_uri: String,
    latest_token: Mutex<Option<CachedToken>>,
}

impl TwitchAuthState {
    async fn fetch_new_token(&self) -> Result<TwitchClientCredentialsGrantResponse, ApiError> {
        let client = reqwest::Client::new();

        let resp = client
            .post(&self.grant_uri)
            .form(&self.grant_request)
            .send()
            .await
            .map_err(|e| ApiError::new(502, &format!("Failed to request twitch token: {e}")))?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(ApiError::new(
                502,
                &format!("Failed to request twitch token ({status}): {body}"),
            ));
        }

        let response: TwitchClientCredentialsGrantResponse = resp.json().await.map_err(|e| {
            ApiError::new(500, &format!("Failed to parse twitch token response: {e}"))
        })?;

        Ok(response)
    }

    async fn fetch_and_cache_token(&self) -> Result<String, ApiError> {
        let response = self.fetch_new_token().await?;

        let expires_at =
            Instant::now() + Duration::from_secs(response.expires_in.saturating_sub(60));

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
            if let Some(cached) = &*mutex {
                if cached.expires_at > Instant::now() {
                    return Ok(cached.access_token.clone());
                }
            }
        }

        self.fetch_and_cache_token().await
    }

    pub async fn new() -> Option<Self> {
        let client_id = get_optional_secret("TWITCH_OAUTH_CLIENT_ID")?;
        let client_secret = get_optional_secret("TWITCH_OAUTH_CLIENT_SECRET")?;
        let grant_uri = std::env::var("TWITCH_OAUTH_TOKEN_URI")
            .unwrap_or_else(|_| "https://id.twitch.tv/oauth2/token".to_string());

        let grant_request = vec![
            ("client_id", client_id.clone()),
            ("client_secret", client_secret),
            ("grant_type", "client_credentials".to_string()),
        ];

        Some(TwitchAuthState {
            client_id,
            grant_request,
            grant_uri,
            latest_token: Mutex::new(None),
        })
    }
}
