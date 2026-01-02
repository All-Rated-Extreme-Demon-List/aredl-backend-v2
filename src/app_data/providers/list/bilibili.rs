use async_trait::async_trait;
use regex::Regex;

use super::super::model::{Provider, ProviderId, ProviderUsage};

pub struct BiliBiliProvider {
    patterns: Vec<Regex>,
}

impl BiliBiliProvider {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // https://bilibili.com/video/<id>
                Regex::new(r"^https?://(?:www\.)?bilibili\.com/video/(?P<id>[A-Za-z0-9]+)")
                    .unwrap(),
            ],
        }
    }
}

#[async_trait]
impl Provider for BiliBiliProvider {
    fn id(&self) -> ProviderId {
        ProviderId::BiliBili
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
        format!("https://www.bilibili.com/video/{}", content_id)
    }
}
