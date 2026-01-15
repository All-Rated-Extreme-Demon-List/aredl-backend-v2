use async_trait::async_trait;
use chrono::{DateTime, TimeZone, Utc};
use regex::Regex;
use serde_json::Value;

use crate::error_handler::ApiError;

use super::super::{
    context::ProviderContext,
    model::{ContentMetadata, Provider, ProviderId, ProviderMatch, ProviderUsage},
};

pub struct MedalProvider {
    patterns: Vec<Regex>,
}

impl MedalProvider {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // https://medal.tv/clips/<id>
                // https://medal.tv/pl/clips/<id>
                Regex::new(
                    r"^https?://medal\.tv(?:/[a-z]{2})?/clips/(?P<id>[A-Za-z0-9_-]+)(?:[/?#].*)?$"
                ).unwrap(),

                // https://medal.tv/games/<game>/clips/<id>
                // https://medal.tv/pl/games/<game>/clips/<id>
                Regex::new(
                    r"^https?://medal\.tv(?:/[a-z]{2})?/games/[A-Za-z0-9_-]+/clips/(?P<id>[A-Za-z0-9_-]+)(?:[/?#].*)?$"
                ).unwrap(),
            ],
        }
    }
}

#[async_trait]
impl Provider for MedalProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Medal
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
        _timestamp: Option<&str>,
        _other_id: Option<&str>,
    ) -> String {
        format!("https://medal.tv/clips/{}", content_id)
    }

    async fn fetch_metadata(
        &self,
        matched: &ProviderMatch,
        context: &ProviderContext,
    ) -> Result<Option<ContentMetadata>, ApiError> {
        let medal_base = std::env::var("MEDAL_API_BASE_URL")
            .unwrap_or_else(|_| "https://medal.tv/api".to_string());

        let url = format!("{}/content/{}", medal_base, matched.content_id);

        let response = context
            .http
            .get(&url)
            .send()
            .await
            .map_err(|e| ApiError::new(502, &format!("Medal API error: {e}")))?;

        if !response.status().is_success() {
            return Err(ApiError::new(
                response.status().as_u16(),
                "Medal API returned non-success",
            ));
        }

        let json: Value = response
            .json()
            .await
            .map_err(|e| ApiError::new(500, &format!("Failed to parse Medal response: {e}")))?;

        let created_ms = json.get("created").and_then(|v| v.as_i64()).or_else(|| {
            json.get("created")
                .and_then(|v| v.as_u64())
                .map(|u| u as i64)
        });

        let published_at: Option<DateTime<Utc>> = created_ms.and_then(|ms| {
            let secs = ms / 1000;
            let nsec = ((ms % 1000) * 1_000_000) as u32;
            Utc.timestamp_opt(secs, nsec).single()
        });

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
