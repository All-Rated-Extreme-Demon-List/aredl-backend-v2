use crate::auth::oauth::OAuthClientConfig;
use crate::auth::oauth::OAuthProvider;
use crate::get_optional_secret;
use crate::providers::context::backend_oauth::{
    BackendGrantType, BackendTokenState, OAuthProviderContext,
};

pub async fn new_patreon_context() -> Option<OAuthProviderContext> {
    let client_secret = get_optional_secret("PATREON_OAUTH_CLIENT_CONFIG")?;

    let mut config: OAuthClientConfig = match serde_json::from_str(&client_secret) {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to parse PATREON_OAUTH_CLIENT_CONFIG: {}", e);
            return None;
        }
    };

    if config.scopes.is_empty() {
        config.scopes.push("identity".to_owned());
    }
    config
        .return_path
        .get_or_insert("/auth/patreon/callback".to_owned());
    config.use_pkce.get_or_insert(false);

    match OAuthProviderContext::new(
        OAuthProvider::Patreon,
        config,
        "https://www.patreon.com/api".to_owned(),
        Some(BackendTokenState::new(BackendGrantType::RefreshToken)),
    ) {
        Ok(context) => Some(context),
        Err(e) => {
            tracing::warn!("Failed to create Patreon OAuth context: {}", e);
            None
        }
    }
}
