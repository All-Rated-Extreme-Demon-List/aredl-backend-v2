pub mod backend_oauth;
pub mod discord;
pub mod google;
pub mod patreon;
pub mod twitch;

use std::sync::Arc;

use crate::{app_data::db::DbAppState, providers::context::backend_oauth::OAuthProviderContext};

#[cfg(test)]
pub(crate) use backend_oauth::decrypt_db_token_value;

#[derive(Clone)]
pub struct ProviderContext {
    pub http: reqwest::Client,
    pub db: Option<Arc<DbAppState>>,
    pub discord_auth: Option<Arc<OAuthProviderContext>>,
    pub google_auth: Option<Arc<OAuthProviderContext>>,
    pub patreon_auth: Option<Arc<OAuthProviderContext>>,
    pub twitch_auth: Option<Arc<OAuthProviderContext>>,
}
