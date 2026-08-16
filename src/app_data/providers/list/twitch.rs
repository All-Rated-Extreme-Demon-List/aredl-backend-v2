use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;
use url::Url;

use crate::{
    error_handler::ApiError,
    providers::model::{ContentMetadata, ProviderMatch},
};

use super::super::parse::{is_ascii_digits, is_twitch_timestamp};
use super::super::{
    context::ProviderContext,
    model::{NormalizedProviderMatch, Provider, ProviderId, ProviderUsage},
};

pub struct TwitchProvider;

#[async_trait]
impl Provider for TwitchProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Twitch
    }

    fn usage(&self) -> ProviderUsage {
        ProviderUsage::Both
    }

    fn hosts(&self) -> &'static [&'static str] {
        &["twitch.tv", "www.twitch.tv", "player.twitch.tv"]
    }

    fn match_url(&self, url: &Url) -> Option<ProviderMatch> {
        let host = url.host_str()?;
        let path = url.path().trim_matches('/');

        let mut timestamp: Option<String> = None;
        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "t" | "time" if timestamp.is_none() => timestamp = Some(value.into_owned()),
                _ => {}
            }
        }

        let content_id: String = if host == "player.twitch.tv" {
            // ?video=v<id>
            let mut player_video_id: Option<String> = None;
            for (key, value) in url.query_pairs() {
                if key.as_ref() == "video" {
                    if let Some(stripped) = value.strip_prefix('v') {
                        player_video_id = Some(stripped.to_owned());
                    }
                    break;
                }
            }
            player_video_id?
        } else {
            // /videos/<id>
            // /<channel>/video/<id>
            // /<channel>/v/<id>
            let mut path_parts = path.split('/');
            match (path_parts.next(), path_parts.next(), path_parts.next()) {
                (Some("videos"), Some(id), _) => id.to_owned(),
                (Some(_channel), Some("video"), Some(id)) => id.to_owned(),
                (Some(_channel), Some("v"), Some(id)) => id.to_owned(),
                _ => return None,
            }
        };

        if !is_ascii_digits(&content_id, 1, 30) {
            return None;
        }

        let timestamp = timestamp
            .map(|value| value.trim().to_owned())
            .filter(|value| is_twitch_timestamp(value));

        Some(ProviderMatch {
            provider: ProviderId::Twitch,
            content_id,
            timestamp,
            other_id: None,
        })
    }

    fn normalize_url(&self, _raw_url: &Url, matched: &ProviderMatch) -> String {
        let mut normalized = format!("https://www.twitch.tv/videos/{}", matched.content_id);

        if let Some(timestamp) = matched.timestamp.as_deref().filter(|s| !s.is_empty()) {
            normalized.push_str("?t=");
            normalized.push_str(timestamp);
        }

        normalized
    }

    async fn fetch_metadata(
        &self,
        matched: &NormalizedProviderMatch,
        context: &ProviderContext,
    ) -> Result<Option<ContentMetadata>, ApiError> {
        let twitch_auth = context
            .twitch_auth
            .as_ref()
            .ok_or_else(|| ApiError::ServiceUnavailable("Twitch support isn't available"))?;
        let db = context
            .db
            .as_ref()
            .ok_or_else(|| ApiError::ServiceUnavailable("Twitch token storage isn't available"))?;

        let access_token = twitch_auth.get_access_token(db).await?;

        let url = format!(
            "{}/videos?id={}",
            twitch_auth.api_base_uri, matched.content_id
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {access_token}"))
                .map_err(|_err| ApiError::InternalServerError("Invalid Twitch access token"))?,
        );
        headers.insert(
            "Client-Id",
            HeaderValue::from_str(&twitch_auth.config.client_id)
                .map_err(|_err| ApiError::InternalServerError("Invalid Twitch client id"))?,
        );

        let response = context
            .http
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| ApiError::BadGateway(format!("Twitch API error: {e}")))?;

        if !response.status().is_success() {
            tracing::warn!("full Twitch API response: {:?}", response);
            return Err(ApiError::BadGateway("Twitch API returned non-success"));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| ApiError::BadGateway(format!("Failed to parse Twitch response: {e}")))?;

        let first = json
            .get("data")
            .and_then(|v| v.as_array())
            .and_then(|arr| arr.first());

        let published_at = first
            .and_then(|v| v.get("published_at"))
            .and_then(|v| v.as_str())
            .and_then(|s| s.parse::<DateTime<Utc>>().ok());

        let Some(_) = first else {
            return Ok(None);
        };

        Ok(Some(ContentMetadata {
            provider: ProviderId::Twitch,
            video_id: matched.content_id.clone(),
            published_at,
        }))
    }
}
