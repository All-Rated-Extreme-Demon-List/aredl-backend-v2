use async_trait::async_trait;
use chrono::{DateTime, Utc};
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value;

use crate::{error_handler::ApiError, providers::model::ContentMetadata};

use super::super::{
    context::ProviderContext,
    model::{Provider, ProviderId, ProviderMatch, ProviderUsage},
};

pub struct TwitchProvider {
    patterns: Vec<Regex>,
}

impl TwitchProvider {
    pub fn new() -> Self {
        Self {
             patterns: vec![
                // https://www.twitch.tv/videos/<id>[?...][t=...]
                Regex::new(
                    r"^https?://(?:www\.)?twitch\.tv/videos/(?P<id>\d+)(?:\?(?:(?:[^#]*?&)?t=(?P<ts>[^&#]+)[^#]*)?[^#]*)?(?:[&#].*)?$"
                ).unwrap(),
                // https://www.twitch.tv/<channel>/video/<id>[?...][t=...]
                Regex::new(
                    r"^https?://(?:www\.)?twitch\.tv/(?P<other>[A-Za-z0-9_]+)/video/(?P<id>\d+)(?:\?(?:(?:[^#]*?&)?t=(?P<ts>[^&#]+)[^#]*)?[^#]*)?(?:[&#].*)?$"
                ).unwrap(),
                // https://www.twitch.tv/<channel>/v/<id>[?...][t=...]
                Regex::new(
                    r"^https?://(?:www\.)?twitch\.tv/(?P<other>[A-Za-z0-9_]+)/v/(?P<id>\d+)(?:\?(?:(?:[^#]*?&)?t=(?P<ts>[^&#]+)[^#]*)?[^#]*)?(?:[&#].*)?$"
                ).unwrap(),
                // https://player.twitch.tv/?video=v<id>&time=...
                Regex::new(
                    r"^https?://player\.twitch\.tv/\?(?:[^#&]*&)*video=v(?P<id>\d+)(?:&(?:[^#&]*&)*time=(?P<ts>[^&#]+))?(?:[&#].*)?$"
                ).unwrap(),
            ],
        }
    }
}

#[async_trait]
impl Provider for TwitchProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Twitch
    }

    fn usage(&self) -> ProviderUsage {
        ProviderUsage::Both
    }

    fn patterns(&self) -> &[Regex] {
        &self.patterns
    }

    fn normalize_url(
        &self,
        _raw_url: &str,
        content_id: &str,
        timestamp: Option<&str>,
        _channel_id: Option<&str>,
    ) -> String {
        match timestamp {
            Some(t) if !t.is_empty() => {
                format!("https://www.twitch.tv/videos/{}?t={}", content_id, t)
            }
            _ => format!("https://www.twitch.tv/videos/{}", content_id),
        }
    }

    async fn fetch_metadata(
        &self,
        matched: &ProviderMatch,
        context: &ProviderContext,
    ) -> Result<Option<ContentMetadata>, ApiError> {
        let twitch_auth = context
            .twitch_auth
            .as_ref()
            .ok_or_else(|| ApiError::new(500, "Twitch support isn't available"))?;

        let access_token = twitch_auth.get_access_token().await?;

        let twitch_base = std::env::var("TWITCH_API_BASE_URL")
            .unwrap_or_else(|_| "https://api.twitch.tv/helix".to_string());

        let url = format!("{}/videos?id={}", twitch_base, matched.content_id);

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", access_token))
                .map_err(|_| ApiError::new(500, "Invalid Twitch access token"))?,
        );
        headers.insert(
            "Client-Id",
            HeaderValue::from_str(&twitch_auth.client_id)
                .map_err(|_| ApiError::new(500, "Invalid Twitch client id"))?,
        );

        let response = context
            .http
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| ApiError::new(502, &format!("Twitch API error: {e}")))?;

        if !response.status().is_success() {
            tracing::warn!("full Twitch API response: {:?}", response);
            return Err(ApiError::new(
                response.status().as_u16(),
                "Twitch API returned non-success",
            ));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| ApiError::new(500, &format!("Failed to parse Twitch response: {e}")))?;

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
