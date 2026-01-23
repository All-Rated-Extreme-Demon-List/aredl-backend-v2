use async_trait::async_trait;
use regex::Regex;
use reqwest::header::HeaderMap;
use serde_json::Value as JsonValue;
use url::Url;

use super::super::{
    context::ProviderContext,
    model::{ContentMetadata, NormalizedProviderMatch, Provider, ProviderId, ProviderUsage},
};
use crate::{error_handler::ApiError, providers::model::ProviderMatch};

pub struct YouTubeProvider;

#[async_trait]
impl Provider for YouTubeProvider {
    fn id(&self) -> ProviderId {
        ProviderId::YouTube
    }

    fn usage(&self) -> ProviderUsage {
        ProviderUsage::CompletionVideo
    }

    fn hosts(&self) -> &'static [&'static str] {
        &[
            "youtube.com",
            "www.youtube.com",
            "m.youtube.com",
            "youtu.be",
        ]
    }

    fn match_url(&self, url: &Url) -> Option<ProviderMatch> {
        let host = url.host_str()?;
        let path = url.path();

        let mut video_id: Option<String> = None;
        let mut timestamp: Option<String> = None;
        let mut list_id: Option<String> = None;

        for (key, value) in url.query_pairs() {
            match key.as_ref() {
                "v" if video_id.is_none() => video_id = Some(value.into_owned()),
                "t" if timestamp.is_none() => timestamp = Some(value.into_owned()),
                "start" if timestamp.is_none() => timestamp = Some(value.into_owned()),
                "list" if list_id.is_none() => list_id = Some(value.into_owned()),
                _ => {}
            }
        }

        let content_id: Option<String> = if host == "youtu.be" {
            url.path()
                .trim_matches('/')
                .split('/')
                .next()
                .map(|s| s.to_string())
        } else if path == "/watch" {
            video_id
        } else {
            let p = path.trim_matches('/');
            let mut it = p.split('/');
            match (it.next(), it.next()) {
                (Some("shorts"), Some(id)) => Some(id.to_string()),
                (Some("live"), Some(id)) => Some(id.to_string()),
                _ => None,
            }
        };

        let content_id = content_id?;
        if !Regex::new(r"^[A-Za-z0-9_-]{11}$")
            .unwrap()
            .is_match(&content_id)
        {
            return None;
        }

        let timestamp = timestamp.map(|s| s.trim().to_string()).filter(|s| {
            !s.is_empty()
                && Regex::new(r"^(?:\d{1,10}|\d{1,4}h)?(?:\d{1,4}m)?(?:\d{1,4}s)?$")
                    .unwrap()
                    .is_match(s)
        });

        let other_id = list_id.map(|s| s.trim().to_string()).filter(|s| {
            !s.is_empty() && Regex::new(r"^[A-Za-z0-9_-]{1,256}$").unwrap().is_match(s)
        });

        Some(ProviderMatch {
            provider: ProviderId::YouTube,
            content_id,
            timestamp,
            other_id,
        })
    }

    fn normalize_url(&self, _raw_url: &Url, matched: &ProviderMatch) -> String {
        let mut normalized =
            Url::parse("https://www.youtube.com/watch").expect("static url is valid");
        {
            let mut query_params = normalized.query_pairs_mut();
            query_params.append_pair("v", &matched.content_id);
            if let Some(ts) = matched.timestamp.as_ref().filter(|s| !s.is_empty()) {
                query_params.append_pair("t", ts);
            }
            if let Some(list) = matched.other_id.as_ref().filter(|s| !s.is_empty()) {
                query_params.append_pair("list", list);
            }
        }
        normalized.to_string()
    }

    async fn fetch_metadata(
        &self,
        matched: &NormalizedProviderMatch,
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

        let youtube_base = std::env::var("YOUTUBE_API_BASE_URL")
            .unwrap_or_else(|_| "https://www.googleapis.com/youtube/v3".to_string());

        let url = format!(
            "{}/videos?part=snippet&id={}",
            youtube_base, matched.content_id
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
