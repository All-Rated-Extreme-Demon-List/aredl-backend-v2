use async_trait::async_trait;
use regex::Regex;

use super::super::model::{Provider, ProviderId, ProviderUsage};

pub struct MegaProvider {
    patterns: Vec<Regex>,
}

impl MegaProvider {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // https://mega.nz/file/<id>#<key>
                Regex::new(
                    r"^https?://mega\.nz/file/(?P<id>[A-Za-z0-9_-]+)#(?P<other>[A-Za-z0-9_-]+)",
                )
                .unwrap(),
                // https://mega.nz/#!<id>!<key>
                Regex::new(
                    r"^https?://mega\.nz/#!(?P<id>[A-Za-z0-9_-]+)!(?P<other>[A-Za-z0-9_-]+)",
                )
                .unwrap(),
            ],
        }
    }
}

#[async_trait]
impl Provider for MegaProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Mega
    }

    fn usage(&self) -> ProviderUsage {
        ProviderUsage::RawFootage
    }

    fn patterns(&self) -> &[Regex] {
        &self.patterns
    }

    fn normalize_url(
        &self,
        _raw_url: &str,
        content_id: &str,
        _timestamp: Option<&str>,
        other_id: Option<&str>,
    ) -> String {
        match other_id {
            Some(key) => format!("https://mega.nz/file/{}#{}", content_id, key),
            None => format!("https://mega.nz/file/{}", content_id),
        }
    }
}
