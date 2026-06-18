use crate::auth::oauth::OAuthClientConfig;
use crate::external_connections::OAuthProvider;
use crate::get_optional_secret;
use crate::providers::context::backend_oauth::OAuthProviderContext;
use crate::providers::context::backend_oauth::{BackendGrantType, BackendTokenState};

pub async fn new_twitch_context() -> Option<OAuthProviderContext> {
    let config_secret = get_optional_secret("TWITCH_OAUTH_CLIENT_CONFIG")?;
    let config: OAuthClientConfig = match serde_json::from_str(&config_secret) {
        Ok(config) => config,
        Err(e) => {
            tracing::warn!("Failed to parse TWITCH_OAUTH_CLIENT_CONFIG: {}", e);
            return None;
        }
    };

    match OAuthProviderContext::new_backend_only(
        OAuthProvider::Twitch,
        config,
        "https://api.twitch.tv/helix".to_string(),
        BackendTokenState::new(BackendGrantType::ClientCredentials),
    ) {
        Ok(context) => Some(context),
        Err(e) => {
            tracing::warn!("Failed to create Twitch OAuth context: {}", e);
            None
        }
    }
}
