use async_trait::async_trait;
use regex::Regex;

use super::super::model::{Provider, ProviderId, ProviderUsage};

pub struct OutplayedProvider {
    patterns: Vec<Regex>,
}

impl OutplayedProvider {
    pub fn new() -> Self {
        Self {
            patterns: vec![Regex::new(
                // https://outplayed.tv/<game>/<id>
                r"^https?://outplayed\.tv/[A-Za-z0-9_-]+/(?P<id>[A-Za-z0-9_-]+)",
            )
            .unwrap()],
        }
    }
}

#[async_trait]
impl Provider for OutplayedProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Outplayed
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
        format!("https://outplayed.tv/media/{}", content_id)
    }
}
