use crate::auth::oauth::OAuthClientConfig;
use crate::auth::oauth::OAuthProvider;
use crate::get_optional_secret;
use crate::providers::context::backend_oauth::OAuthProviderContext;

pub async fn new_discord_context() -> Option<OAuthProviderContext> {
    let client_secret = get_optional_secret("DISCORD_OAUTH_CLIENT_CONFIG")?;

    let mut config: OAuthClientConfig = match serde_json::from_str(&client_secret) {
        Ok(config) => config,
        Err(e) => {
            tracing::warn!("Failed to parse DISCORD_OAUTH_CLIENT_CONFIG: {}", e);
            return None;
        }
    };

    if config.scopes.is_empty() {
        config.scopes.push("identify".to_owned());
    }
    config
        .return_path
        .get_or_insert("/auth/discord/callback".to_owned());
    config.use_pkce.get_or_insert(true);

    match OAuthProviderContext::new(
        OAuthProvider::Discord,
        config,
        "https://discord.com".to_owned(),
        None,
    ) {
        Ok(context) => Some(context),
        Err(e) => {
            tracing::warn!("Failed to create Discord OAuth context: {}", e);
            None
        }
    }
}
