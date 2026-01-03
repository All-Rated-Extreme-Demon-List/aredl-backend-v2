use async_trait::async_trait;
use regex::Regex;
use reqwest::header::HeaderMap;
use serde_json::Value as JsonValue;

use super::super::{
    context::ProviderContext,
    model::{ContentMetadata, Provider, ProviderId, ProviderMatch, ProviderUsage},
};
use crate::error_handler::ApiError;

pub struct YouTubeProvider {
    patterns: Vec<Regex>,
}

impl YouTubeProvider {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // https://(www.|m.)youtube.com/watch?v=<id>[...][&t=... or &start=...]
                Regex::new(
                    r"^https?://(?:www\.|m\.)?youtube\.com/watch\?(?:[^#]*?[&?])?v=(?P<id>[A-Za-z0-9_-]{11})(?:[^#]*?[&?](?:t|start)=(?P<ts>[^&#]+))?(?:[&#].*)?$"
                ).unwrap(),
                // https://youtu.be/<id>[...][t=... or start=...]
                Regex::new(
                    r"^https?://youtu\.be/(?P<id>[A-Za-z0-9_-]{11})(?:\?(?:(?:[^#]*?&)?(?:t|start)=(?P<ts>[^&#]+)[^#]*)?[^#]*)?(?:[&#].*)?$"
                ).unwrap(),
                // https://(www.|m.)youtube.com/shorts/<id>[...][t=... or start=...]
                Regex::new(
                    r"^https?://(?:www\.|m\.)?youtube\.com/shorts/(?P<id>[A-Za-z0-9_-]{11})(?:\?(?:(?:[^#]*?&)?(?:t|start)=(?P<ts>[^&#]+)[^#]*)?[^#]*)?(?:[&#].*)?$"
                ).unwrap(),
                // https://(www.|m.)youtube.com/live/<id>[...][t=... or start=...]
                Regex::new(
                    r"^https?://(?:www\.|m\.)?youtube\.com/live/(?P<id>[A-Za-z0-9_-]{11})(?:\?(?:(?:[^#]*?&)?(?:t|start)=(?P<ts>[^&#]+)[^#]*)?[^#]*)?(?:[&#].*)?$"
                ).unwrap(),
            ],
        }
    }
}

#[async_trait]
impl Provider for YouTubeProvider {
    fn id(&self) -> ProviderId {
        ProviderId::YouTube
    }

    fn usage(&self) -> ProviderUsage {
        ProviderUsage::CompletionVideo
    }

    fn patterns(&self) -> &[Regex] {
        &self.patterns
    }

    fn normalize_url(
        &self,
        _raw_url: &str,
        content_id: &str,
        timestamp: Option<&str>,
        _other_id: Option<&str>,
    ) -> String {
        match timestamp {
            Some(timestamp) if !timestamp.is_empty() => {
                format!(
                    "https://www.youtube.com/watch?v={}&t={}",
                    content_id, timestamp
                )
            }
            _ => format!("https://www.youtube.com/watch?v={}", content_id),
        }
    }

    async fn fetch_metadata(
        &self,
        matched: &ProviderMatch,
        context: &ProviderContext,
    ) -> Result<Option<ContentMetadata>, ApiError> {
        let google_auth = context
            .google_auth
            .as_ref()
            .ok_or_else(|| ApiError::new(500, "Youtube support isn't available"))?;

        let token = google_auth
            .get_access_token()
            .await
            .map_err(|e| ApiError::new(502, &format!("Failed to acquire Youtube token: {e}")))?;

        let url = format!(
            "https://www.googleapis.com/youtube/v3/videos?part=snippet&id={}",
            matched.content_id
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );

        let response = context
            .http
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| ApiError::new(502, &format!("YouTube API error: {e}")))?;

        if !response.status().is_success() {
            tracing::warn!("full YouTube API response: {:?}", response);
            return Err(ApiError::new(
                response.status().as_u16(),
                "YouTube API returned non-success",
            ));
        }

        let json: JsonValue = response
            .json()
            .await
            .map_err(|e| ApiError::new(500, &format!("Failed to parse YouTube response: {e}")))?;

        let items = json
            .get("items")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let first = match items.first() {
            Some(v) => v,
            None => return Ok(None),
        };

        let snippet = first.get("snippet").and_then(|v| v.as_object()).cloned();
        let published_at = if let Some(snippet) = snippet {
            snippet
                .get("publishedAt")
                .and_then(|v| v.as_str())
                .and_then(|s: &str| s.parse().ok())
        } else {
            None
        };

        Ok(Some(ContentMetadata {
            provider: ProviderId::YouTube,
            video_id: matched.content_id.clone(),
            published_at,
        }))
    }
}
