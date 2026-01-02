use async_trait::async_trait;
use regex::Regex;

use super::super::model::{Provider, ProviderId, ProviderUsage};

pub struct VimeoProvider {
    patterns: Vec<Regex>,
}

impl VimeoProvider {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // https://vimeo.com/<id>
                Regex::new(r"^https?://(?:www\.)?vimeo\.com/(?P<id>[0-9]+)").unwrap(),
                // https://player.vimeo.com/video/<id>
                Regex::new(r"^https?://player\.vimeo\.com/video/(?P<id>[0-9]+)").unwrap(),
            ],
        }
    }
}

#[async_trait]
impl Provider for VimeoProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Vimeo
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
        format!("https://vimeo.com/{}", content_id)
    }
}
