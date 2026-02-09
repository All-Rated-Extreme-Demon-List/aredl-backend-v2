use async_trait::async_trait;
use chrono::{DateTime, Utc};
use reqwest::header::HeaderMap;
use reqwest::{header, Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use url::Url;

use super::context::ProviderContext;
use crate::error_handler::ApiError;

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash, Serialize, Deserialize)]
pub enum ProviderId {
    YouTube,
    Vimeo,
    Twitch,
    BiliBili,
    Medal,
    Outplayed,
    GoogleDrive,
    Mega,
    Mediafire,
}

#[derive(Debug, Clone, Copy)]
pub enum ProviderUsage {
    CompletionVideo,
    RawFootage,
    Both,
}

impl ProviderUsage {
    pub fn allowed_for_completion(&self) -> bool {
        match self {
            ProviderUsage::CompletionVideo | ProviderUsage::Both => true,
            ProviderUsage::RawFootage => false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ProviderMatch {
    pub provider: ProviderId,
    pub content_id: String,
    pub timestamp: Option<String>,
    pub other_id: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NormalizedProviderMatch {
    pub provider: ProviderId,
    pub content_id: String,
    pub other_id: Option<String>,
    pub normalized_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub provider: ProviderId,
    pub video_id: String,
    pub published_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct ContentDataLocation {
    pub url: String,
    pub headers: HeaderMap,
}

pub struct ProviderRegistry {
    providers: HashMap<ProviderId, Arc<dyn Provider>>,
}

impl ProviderRegistry {
    pub fn new(providers: Vec<Arc<dyn Provider>>) -> Self {
        let mut map = HashMap::new();
        for p in providers {
            map.insert(p.id(), p);
        }
        Self { providers: map }
    }

    pub fn providers(&self) -> impl Iterator<Item = &Arc<dyn Provider>> {
        self.providers.values()
    }

    pub fn get(&self, id: ProviderId) -> Option<Arc<dyn Provider>> {
        self.providers.get(&id).cloned()
    }

    pub fn match_url(&self, url: &Url) -> Result<NormalizedProviderMatch, ApiError> {
        for provider in self.providers.values() {
            match provider.parse_url(url)? {
                Some(matched) => return Ok(matched),
                None => continue,
            }
        }
        Err(ApiError::new(
            422,
            "URL does not match any known supported providers. Please refer to our guidelines for a list of supported websites.",
        ))
    }
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn id(&self) -> ProviderId;
    fn usage(&self) -> ProviderUsage;

    fn hosts(&self) -> &'static [&'static str];

    fn normalize_url(&self, raw_url: &Url, matched: &ProviderMatch) -> String;

    fn match_url(&self, url: &Url) -> Option<ProviderMatch>;

    fn parse_url(&self, url: &Url) -> Result<Option<NormalizedProviderMatch>, ApiError> {
        let host = match url.host_str() {
            Some(h) => h,
            None => return Ok(None),
        };

        if !self.hosts().is_empty() && !self.hosts().iter().any(|&h| h == host) {
            return Ok(None);
        }

        if let Some(matched) = self.match_url(url) {
            let normalized_url = self.normalize_url(url, &matched);
            Ok(Some(NormalizedProviderMatch {
                provider: matched.provider,
                content_id: matched.content_id,
                other_id: matched.other_id,
                normalized_url,
            }))
        } else {
            Ok(None)
        }
    }

    async fn fetch_metadata(
        &self,
        matched: &NormalizedProviderMatch,
        context: &ProviderContext,
    ) -> Result<Option<ContentMetadata>, ApiError> {
        let _ = (matched, context);
        Ok(None)
    }

    async fn get_content_location(
        &self,
        matched: &NormalizedProviderMatch,
        context: &ProviderContext,
    ) -> Result<Option<ContentDataLocation>, ApiError> {
        let _ = (matched, context);
        Ok(None)
    }
}

pub const FETCH_RANGE_CHUNK_SIZE: u64 = 16 * 1024 * 1024;

impl ContentDataLocation {
    async fn fetch_range(&self, start: u64, len: u64) -> Result<Vec<u8>, ApiError> {
        let client = Client::new();
        let end = start
            .checked_add(len)
            .and_then(|v| v.checked_sub(1))
            .ok_or_else(|| ApiError::new(500, "Invalid range parameters"))?;

        let range = format!("bytes={}-{}", start, end);

        let response = client
            .get(&self.url)
            .headers(self.headers.clone())
            .header(header::RANGE, range)
            .send()
            .await
            .map_err(|e| ApiError::new(502, &format!("Failed to request file range: {e}")))?;

        if !response.status().is_success() && response.status() != StatusCode::PARTIAL_CONTENT {
            let status = response.status();
            return Err(ApiError::new(
                status.as_u16(),
                &format!("Failed to request file range: {status}"),
            ));
        }

        let bytes = response
            .bytes()
            .await
            .map_err(|e| ApiError::new(500, &format!("Failed to read file bytes: {e}")))?;

        Ok(bytes.to_vec())
    }

    pub async fn fetch_head(&self) -> Result<Vec<u8>, ApiError> {
        self.fetch_range(0, FETCH_RANGE_CHUNK_SIZE).await
    }

    pub async fn fetch_from_offset(&self, offset: u64) -> Result<Vec<u8>, ApiError> {
        self.fetch_range(offset, FETCH_RANGE_CHUNK_SIZE).await
    }
}
