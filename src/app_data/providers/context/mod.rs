mod google;

use std::sync::Arc;

pub use google::GoogleAuthState;

#[derive(Clone)]
pub struct ProviderContext {
    pub http: reqwest::Client,
    pub google_auth: Option<Arc<GoogleAuthState>>,
}
