use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value;
use url::Url;

use crate::{error_handler::ApiError, providers::model::ProviderMatch};

use super::super::parse::is_ascii_id;
use super::super::{
    context::ProviderContext,
    model::{ContentMetadata, NormalizedProviderMatch, Provider, ProviderId, ProviderUsage},
};

pub struct MedalProvider;

#[async_trait]
impl Provider for MedalProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Medal
    }

    fn usage(&self) -> ProviderUsage {
        ProviderUsage::CompletionVideo
    }

    fn hosts(&self) -> &'static [&'static str] {
        &["medal.tv", "www.medal.tv"]
    }

    fn match_url(&self, url: &Url) -> Option<ProviderMatch> {
        // /clips/<id>
        // /<lang>/clips/<id>
        // /games/<game>/clips/<id>
        // /<lang>/games/<game>/clips/<id>

        let path = url.path().trim_matches('/');

        let mut parts = path.split('/');

        let first = parts.next()?;
        let (_maybe_lang, first) =
            if first.len() == 2 && first.bytes().all(|b| b.is_ascii_lowercase()) {
                (Some(first), parts.next()?)
            } else {
                (None, first)
            };

        let content_id = match first {
            "clips" => parts.next(),
            "games" => {
                let _game = parts.next()?;
                match parts.next()? {
                    "clips" => parts.next(),
                    _ => None,
                }
            }
            _ => None,
        }?;

        if !is_ascii_id(content_id, 1, 128) {
            return None;
        }

        Some(ProviderMatch {
            provider: ProviderId::Medal,
            content_id: content_id.to_owned(),
            timestamp: None,
            other_id: None,
        })
    }

    fn normalize_url(&self, _raw_url: &Url, matched: &ProviderMatch) -> String {
        format!("https://medal.tv/clips/{}", matched.content_id)
    }

    async fn fetch_metadata(
        &self,
        matched: &NormalizedProviderMatch,
        context: &ProviderContext,
    ) -> Result<Option<ContentMetadata>, ApiError> {
        let medal_base = std::env::var("MEDAL_API_BASE_URL")
            .unwrap_or_else(|_| "https://medal.tv/api".to_owned());

        let url = format!("{}/content/{}", medal_base, matched.content_id);

        let response = context
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::BadGateway(format!("Medal API error: {e}")))?;

        if !response.status().is_success() {
            return Err(ApiError::BadGateway("Medal API returned non-success"));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| ApiError::BadGateway(format!("Failed to parse Medal response: {e}")))?;

        let created_ms = json.get("created").and_then(Value::as_i64).or_else(|| {
            let value = json.get("created").and_then(Value::as_u64)?;
            i64::try_from(value).ok()
        });

        let published_at: Option<DateTime<Utc>> =
            created_ms.and_then(DateTime::<Utc>::from_timestamp_millis);

        if published_at.is_none() {
            return Ok(None);
        }

        Ok(Some(ContentMetadata {
            provider: ProviderId::Medal,
            video_id: matched.content_id.clone(),
            published_at,
        }))
    }
}
