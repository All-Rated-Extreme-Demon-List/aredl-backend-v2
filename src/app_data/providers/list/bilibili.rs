use async_trait::async_trait;
use url::Url;

use crate::providers::model::ProviderMatch;

use super::super::model::{Provider, ProviderId, ProviderUsage};
use super::super::parse::is_ascii_alphanumeric;

pub struct BiliBiliProvider;

#[async_trait]
impl Provider for BiliBiliProvider {
    fn id(&self) -> ProviderId {
        ProviderId::BiliBili
    }

    fn usage(&self) -> ProviderUsage {
        ProviderUsage::CompletionVideo
    }

    fn hosts(&self) -> &'static [&'static str] {
        &["bilibili.com", "www.bilibili.com", "m.bilibili.com"]
    }

    fn match_url(&self, url: &Url) -> Option<ProviderMatch> {
        // https://bilibili.com/video/<id>
        let path = url.path().trim_matches('/');

        let mut parts = path.split('/');
        match (parts.next(), parts.next()) {
            (Some("video"), Some(id)) => {
                if !is_ascii_alphanumeric(id) {
                    return None;
                }

                Some(ProviderMatch {
                    provider: ProviderId::BiliBili,
                    content_id: id.to_owned(),
                    timestamp: None,
                    other_id: None,
                })
            }
            _ => None,
        }
    }

    fn normalize_url(&self, _raw_url: &Url, matched: &ProviderMatch) -> String {
        format!("https://www.bilibili.com/video/{}", matched.content_id)
    }
}
