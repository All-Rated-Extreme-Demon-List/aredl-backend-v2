use url::Url;

use super::{
    context::ProviderContext,
    list::{
        bilibili::BiliBiliProvider, gdrive::GoogleDriveProvider, medal::MedalProvider,
        mega::MegaProvider, outplayed::OutplayedProvider, twitch::TwitchProvider,
        vimeo::VimeoProvider, youtube::YouTubeProvider,
    },
    model::{
        ContentDataLocation, ContentMetadata, NormalizedProviderMatch, Provider, ProviderRegistry,
    },
};
use crate::{
    app_data::db::DbAppState,
    error_handler::ApiError,
    providers::context::{
        discord::new_discord_context, google::new_google_context, patreon::new_patreon_context,
        twitch::new_twitch_context,
    },
};

use std::sync::Arc;

pub struct ProvidersAppState {
    registry: ProviderRegistry,
    pub context: ProviderContext,
}

impl ProvidersAppState {
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
            return Err(ApiError::BadRequest("Malformed URL"));
        }
        let url = Url::parse(input)
            .map_err(|error| ApiError::BadRequest(format!("Malformed URL: {}", error)))?;
        Ok(url)
    }

    // completion video enforces a valid url and an allowed provider
    pub fn validate_completion_video_url(&self, url: &str) -> Result<String, ApiError> {
        let matched = self.parse_url(url)?;
        let provider = self
            .registry
            .get(matched.provider)
            .ok_or_else(|| ApiError::InternalServerError("Provider not registered"))?;

        if !provider.usage().allowed_for_completion() {
            return Err(ApiError::UnprocessableEntity(
                "This provider is not allowed for this field",
            ));
        }

        Ok(matched.normalized_url)
    }

    // raw footage only enforces a valid url, but if it matches a provider, normalize it
    pub fn validate_raw_footage_url(&self, url: &str) -> Result<String, ApiError> {
        self.validate_is_url(url)?;

        match self.parse_url(url) {
            Ok(matched) => Ok(matched.normalized_url),
            Err(_) => Ok(url.to_string()),
        }
    }

    pub async fn get_content_location(
        &self,
        matched: &NormalizedProviderMatch,
    ) -> Result<Option<ContentDataLocation>, ApiError> {
        let provider = self
            .registry
            .get(matched.provider)
            .ok_or_else(|| ApiError::InternalServerError("Provider not registered"))?;
        provider.get_content_location(matched, &self.context).await
    }

    pub async fn fetch_metadata(
        &self,
        matched: &NormalizedProviderMatch,
    ) -> Result<Option<ContentMetadata>, ApiError> {
        let provider = self
            .registry
            .get(matched.provider)
            .ok_or_else(|| ApiError::InternalServerError("Provider not registered"))?;
        provider.fetch_metadata(matched, &self.context).await
    }
}

pub async fn init_app_state(db: Arc<DbAppState>) -> Arc<ProvidersAppState> {
    let http = reqwest::Client::new();
    let discord_state = new_discord_context().await.map(Arc::new);
    let google_state = new_google_context().await.map(Arc::new);
    let patreon_state = new_patreon_context().await.map(Arc::new);
    let twitch_state = new_twitch_context().await.map(Arc::new);

    let context = ProviderContext {
        http,
        db: Some(db),
        discord_auth: discord_state,
        google_auth: google_state,
        patreon_auth: patreon_state,
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

    Arc::new(ProvidersAppState::new(registry, context))
}
