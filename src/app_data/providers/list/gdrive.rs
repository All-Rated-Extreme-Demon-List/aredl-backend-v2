use async_trait::async_trait;
use regex::Regex;
use reqwest::header::HeaderMap;

use super::super::{
    context::ProviderContext,
    model::{ContentDataLocation, Provider, ProviderId, ProviderMatch, ProviderUsage},
};
use crate::error_handler::ApiError;

pub struct GoogleDriveProvider {
    patterns: Vec<Regex>,
}

impl GoogleDriveProvider {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // https://drive.google.com/file/d/<id>
                // https://drive.google.com/u/0/file/d/<id>
                Regex::new(
                    r"^https?://drive\.google\.com(?:/u/\d+)?/file/d/(?P<id>[\w-]+)(?:[/?#].*)?$"
                ).unwrap(),
                // https://drive.google.com/drive/folders/<id>
                // https://drive.google.com/u/0/drive/folders/<id>
                Regex::new(
                    r"^https?://drive\.google\.com/drive(?:/u/\d+)?/folders/(?P<id>[\w-]+)(?:[/?#].*)?$"
                ).unwrap(),
                // https://drive.google.com/open?id=<id>
                Regex::new(
                    r"^https?://drive\.google\.com(?:/u/\d+)?/open\?(?:[^#]*?&)?id=(?P<id>[\w-]+)(?:[&#].*)?$"
                ).unwrap(),
                // https://drive.google.com/uc?id=<id>
                Regex::new(
                    r"^https?://drive\.google\.com(?:/u/\d+)?/uc\?(?:[^#]*?&)?id=(?P<id>[\w-]+)(?:[&#].*)?$"
                ).unwrap(),
            ],
        }
    }
}

#[async_trait]
impl Provider for GoogleDriveProvider {
    fn id(&self) -> ProviderId {
        ProviderId::GoogleDrive
    }

    fn usage(&self) -> ProviderUsage {
        ProviderUsage::RawFootage
    }

    fn patterns(&self) -> &[Regex] {
        &self.patterns
    }

    fn normalize_url(
        &self,
        raw_url: &str,
        content_id: &str,
        _timestamp: Option<&str>,
        _other_id: Option<&str>,
    ) -> String {
        if raw_url.contains("/folders/") {
            format!("https://drive.google.com/drive/folders/{}", content_id)
        } else {
            format!("https://drive.google.com/file/d/{}", content_id)
        }
    }

    async fn get_content_location(
        &self,
        matched: &ProviderMatch,
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
