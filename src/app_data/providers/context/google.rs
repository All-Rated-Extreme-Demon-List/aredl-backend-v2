use crate::auth::oauth::OAuthClientConfig;
use crate::auth::oauth::OAuthProvider;
use crate::get_optional_secret;
use crate::providers::context::backend_oauth::{
    BackendGrantType, BackendTokenState, OAuthProviderContext,
};

pub async fn new_google_context() -> Option<OAuthProviderContext> {
    let client_secret = get_optional_secret("GOOGLE_OAUTH_CLIENT_CONFIG")?;

    let config: OAuthClientConfig = match serde_json::from_str(&client_secret) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to parse GOOGLE_OAUTH_CLIENT_CONFIG: {}", e);
            return None;
        }
    };

    match OAuthProviderContext::new_backend_only(
        OAuthProvider::Google,
        config,
        "https://www.googleapis.com".to_string(),
        BackendTokenState::new(BackendGrantType::RefreshToken),
    ) {
        Ok(context) => Some(context),
        Err(e) => {
            tracing::warn!("Failed to create Google OAuth context: {}", e);
            None
        }
    }
}
