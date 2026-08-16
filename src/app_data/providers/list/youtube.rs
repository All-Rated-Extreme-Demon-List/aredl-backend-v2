use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue};
use serde_json::Value as JsonValue;
use url::form_urlencoded::Serializer;
use url::Url;

use super::super::parse::{is_ascii_id, is_youtube_timestamp};
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
                "t" | "start" if timestamp.is_none() => timestamp = Some(value.into_owned()),
                "list" if list_id.is_none() => list_id = Some(value.into_owned()),
                _ => {}
            }
        }

        let content_id: Option<String> = if host == "youtu.be" {
            url.path()
                .trim_matches('/')
                .split('/')
                .next()
                .map(str::to_owned)
        } else if path == "/watch" {
            video_id
        } else {
            let p = path.trim_matches('/');
            let mut it = p.split('/');
            match (it.next(), it.next()) {
                (Some("shorts" | "live"), Some(id)) => Some(id.to_owned()),
                _ => None,
            }
        };

        let content_id = content_id?;
        if !is_ascii_id(&content_id, 11, 11) {
            return None;
        }

        let timestamp = timestamp
            .map(|s| s.trim().to_owned())
            .filter(|s| is_youtube_timestamp(s));

        let other_id = list_id
            .map(|s| s.trim().to_owned())
            .filter(|s| is_ascii_id(s, 1, 256));

        Some(ProviderMatch {
            provider: ProviderId::YouTube,
            content_id,
            timestamp,
            other_id,
        })
    }

    fn normalize_url(&self, _raw_url: &Url, matched: &ProviderMatch) -> String {
        let mut query = Serializer::new(String::new());
        query.append_pair("v", &matched.content_id);

        if let Some(ts) = matched.timestamp.as_ref().filter(|s| !s.is_empty()) {
            query.append_pair("t", ts);
        }
        if let Some(list) = matched.other_id.as_ref().filter(|s| !s.is_empty()) {
            query.append_pair("list", list);
        }

        format!("https://www.youtube.com/watch?{}", query.finish())
    }

    async fn fetch_metadata(
        &self,
        matched: &NormalizedProviderMatch,
        context: &ProviderContext,
    ) -> Result<Option<ContentMetadata>, ApiError> {
        let google_auth = context
            .google_auth
            .as_ref()
            .ok_or_else(|| ApiError::ServiceUnavailable("Youtube support isn't available"))?;
        let db = context
            .db
            .as_ref()
            .ok_or_else(|| ApiError::ServiceUnavailable("Google token storage isn't available"))?;

        let token = google_auth
            .get_access_token(db)
            .await
            .map_err(|e| ApiError::BadGateway(format!("Failed to acquire Youtube token: {e}")))?;

        let url = format!(
            "{}/youtube/v3/videos?part=snippet&id={}",
            google_auth.api_base_uri.trim_end_matches('/'),
            matched.content_id
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {token}"))
                .map_err(|_err| ApiError::InternalServerError("Invalid Youtube access token"))?,
        );

        let response = context
            .http
            .get(&url)
            .headers(headers)
            .send()
            .await
            .map_err(|e| ApiError::BadGateway(format!("YouTube API error: {e}")))?;

        if !response.status().is_success() {
            return Err(ApiError::BadGateway("YouTube API returned non-success"));
        }

        let json: JsonValue = response
            .json()
            .await
            .map_err(|e| ApiError::BadGateway(format!("Failed to parse YouTube response: {e}")))?;

        let items: &[JsonValue] = json
            .get("items")
            .and_then(|v| v.as_array())
            .map_or(&[], Vec::as_slice);

        let Some(first) = items.first() else {
            return Ok(None);
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
