use std::sync::Arc;
use openidconnect::core::CoreClient;

use crate::auth::discord::create_discord_client;

pub struct AuthAppState {
    pub discord_client: CoreClient,
}

pub async fn init_app_state() -> Arc<AuthAppState> {
    let discord_client = create_discord_client().await.expect("Failed to create discord client!");
    Arc::new(AuthAppState {discord_client})
}