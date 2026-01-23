mod google;
mod twitch;

use std::sync::Arc;

pub use google::GoogleAuthState;
pub use twitch::TwitchAuthState;

#[derive(Clone)]
pub struct ProviderContext {
    pub http: reqwest::Client,
    pub google_auth: Option<Arc<GoogleAuthState>>,
    pub twitch_auth: Option<Arc<TwitchAuthState>>,
}
