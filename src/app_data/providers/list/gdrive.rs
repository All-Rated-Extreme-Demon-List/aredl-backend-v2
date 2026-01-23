use async_trait::async_trait;
use regex::Regex;
use reqwest::header::HeaderMap;
use url::Url;

use super::super::{
    context::ProviderContext,
    model::{ContentDataLocation, NormalizedProviderMatch, Provider, ProviderId, ProviderUsage},
};
use crate::{error_handler::ApiError, providers::model::ProviderMatch};

pub struct GoogleDriveProvider;

#[async_trait]
impl Provider for GoogleDriveProvider {
    fn id(&self) -> ProviderId {
        ProviderId::GoogleDrive
    }

    fn usage(&self) -> ProviderUsage {
        ProviderUsage::RawFootage
    }

    fn hosts(&self) -> &'static [&'static str] {
        &["drive.google.com"]
    }

    fn match_url(&self, url: &Url) -> Option<ProviderMatch> {
        let path = url.path().trim_matches('/');
        let mut parts = path.split('/').peekable();

        // ignore u/<n> or drive/ prefixes
        loop {
            match parts.peek().copied() {
                Some("drive") => {
                    parts.next();
                }
                Some("u") => {
                    parts.next();
                    let _ = parts.next()?;
                }
                _ => break,
            }
        }

        let (content_id, is_folder) = match (parts.next(), parts.next(), parts.next()) {
            (Some("file"), Some("d"), Some(id)) => (id.to_string(), false),
            (Some("folders"), Some(id), _) => (id.to_string(), true),
            _ => {
                // open / uc
                match url.path() {
                    "/open" | "/uc" => {
                        let mut query_id: Option<String> = None;
                        for (key, value) in url.query_pairs() {
                            if key == "id" {
                                query_id = Some(value.into_owned());
                                break;
                            }
                        }
                        (query_id?, false)
                    }
                    _ => return None,
                }
            }
        };

        if !Regex::new(r"^[A-Za-z0-9_-]+$")
            .unwrap()
            .is_match(&content_id)
        {
            return None;
        }

        Some(ProviderMatch {
            provider: ProviderId::GoogleDrive,
            content_id,
            timestamp: None,
            other_id: if is_folder {
                Some("folder".to_string())
            } else {
                None
            },
        })
    }

    fn normalize_url(&self, _raw_url: &Url, matched: &ProviderMatch) -> String {
        if matched.other_id.as_deref() == Some("folder") {
            format!(
                "https://drive.google.com/drive/folders/{}",
                matched.content_id
            )
        } else {
            format!("https://drive.google.com/file/d/{}", matched.content_id)
        }
    }

    async fn get_content_location(
        &self,
        matched: &NormalizedProviderMatch,
        context: &ProviderContext,
    ) -> Result<Option<ContentDataLocation>, ApiError> {
        let google_auth = context
            .google_auth
            .as_ref()
            .ok_or_else(|| ApiError::new(500, "Google Drive support isn't available"))?;

        let token = google_auth
            .get_access_token()
            .await
            .map_err(|e| ApiError::new(502, &format!("Failed to acquire Drive token: {e}")))?;

        let url = format!(
            "https://www.googleapis.com/drive/v3/files/{}?alt=media&supportsAllDrives=true",
            matched.content_id
        );

        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            reqwest::header::HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );

        Ok(Some(ContentDataLocation { url, headers }))
    }
}
