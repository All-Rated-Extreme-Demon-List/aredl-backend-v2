use url::Url;

use super::{
    context::{GoogleAuthState, ProviderContext},
    list::{
        bilibili::BiliBiliProvider, gdrive::GoogleDriveProvider, medal::MedalProvider,
        mega::MegaProvider, outplayed::OutplayedProvider, twitch::TwitchProvider,
        vimeo::VimeoProvider, youtube::YouTubeProvider,
    },
    model::{
        ContentDataLocation, ContentMetadata, NormalizedProviderMatch, Provider, ProviderRegistry,
    },
};
use crate::{error_handler::ApiError, providers::context::TwitchAuthState};

use std::sync::Arc;

pub struct VideoProvidersAppState {
    registry: ProviderRegistry,
    context: ProviderContext,
}

impl VideoProvidersAppState {
    pub fn new(registry: ProviderRegistry, context: ProviderContext) -> Self {
        Self { registry, context }
    }

    pub fn parse_url(&self, url: &str) -> Result<NormalizedProviderMatch, ApiError> {
        let url = self.validate_is_url(url)?;
        self.registry.match_url(&url)
    }

    pub fn validate_is_url(&self, url: &str) -> Result<Url, ApiError> {
        let input = url.trim();
        if input.chars().any(|char| char.is_whitespace()) {
            return Err(ApiError::new(400, "Malformed URL"));
        }
        let url = Url::parse(input).map_err(|_| ApiError::new(400, "Malformed URL"))?;
        Ok(url)
    }

    // completion video enforces a valid url and an allowed provider
    pub fn validate_completion_video_url(&self, url: &str) -> Result<String, ApiError> {
        let matched = self.parse_url(&url)?;
        let provider = self
            .registry
            .get(matched.provider)
            .ok_or_else(|| ApiError::new(500, "Provider not registered"))?;

        if !provider.usage().allowed_for_completion() {
            return Err(ApiError::new(
                422,
                "This provider is not allowed for this field",
            ));
        }

        Ok(matched.normalized_url)
    }

    // raw footage only enforces a valid url, but if it matches a provider, normalize it
    pub fn validate_raw_footage_url(&self, url: &str) -> Result<String, ApiError> {
        self.validate_is_url(url)?;
        let matched = self.parse_url(url);

        if matched.is_err() {
            return Ok(url.to_string());
        }

        Ok(matched.unwrap().normalized_url)
    }

    pub async fn get_content_location(
        &self,
        matched: &NormalizedProviderMatch,
    ) -> Result<Option<ContentDataLocation>, ApiError> {
        let provider = self
            .registry
            .get(matched.provider)
            .ok_or_else(|| ApiError::new(500, "Provider not registered"))?;
        provider.get_content_location(matched, &self.context).await
    }

    pub async fn fetch_metadata(
        &self,
        matched: &NormalizedProviderMatch,
    ) -> Result<Option<ContentMetadata>, ApiError> {
        let provider = self
            .registry
            .get(matched.provider)
            .ok_or_else(|| ApiError::new(500, "Provider not registered"))?;
        provider.fetch_metadata(matched, &self.context).await
    }
}

pub async fn init_app_state() -> Arc<VideoProvidersAppState> {
    let http = reqwest::Client::new();
    let google_state = GoogleAuthState::new().await.map(Arc::new);
    let twitch_state = TwitchAuthState::new().await.map(Arc::new);

    let context = ProviderContext {
        http,
        google_auth: google_state,
        twitch_auth: twitch_state,
    };

    let registry = ProviderRegistry::new(vec![
        Arc::new(YouTubeProvider) as Arc<dyn Provider>,
        Arc::new(TwitchProvider) as Arc<dyn Provider>,
        Arc::new(VimeoProvider) as Arc<dyn Provider>,
        Arc::new(MedalProvider) as Arc<dyn Provider>,
        Arc::new(BiliBiliProvider) as Arc<dyn Provider>,
        Arc::new(OutplayedProvider) as Arc<dyn Provider>,
        Arc::new(GoogleDriveProvider) as Arc<dyn Provider>,
        Arc::new(MegaProvider) as Arc<dyn Provider>,
    ]);

    Arc::new(VideoProvidersAppState::new(registry, context))
}
