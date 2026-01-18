use async_trait::async_trait;
use regex::Regex;
use url::Url;

use crate::providers::model::ProviderMatch;

use super::super::model::{Provider, ProviderId, ProviderUsage};

pub struct OutplayedProvider;

#[async_trait]
impl Provider for OutplayedProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Outplayed
    }

    fn usage(&self) -> ProviderUsage {
        ProviderUsage::CompletionVideo
    }

    fn hosts(&self) -> &'static [&'static str] {
        &["outplayed.tv", "www.outplayed.tv"]
    }

    fn match_url(&self, url: &Url) -> Option<ProviderMatch> {
        // https://outplayed.tv/<game>/<id>
        let path = url.path().trim_matches('/');
        let mut parts = path.split('/');

        let _game = parts.next()?;
        let content_id = parts.next()?;

        if !Regex::new(r"^[A-Za-z0-9_-]{1,128}$")
            .unwrap()
            .is_match(content_id)
        {
            return None;
        }

        Some(ProviderMatch {
            provider: ProviderId::Outplayed,
            content_id: content_id.to_string(),
            timestamp: None,
            other_id: None,
        })
    }

    fn normalize_url(&self, _raw_url: &Url, matched: &ProviderMatch) -> String {
        format!("https://outplayed.tv/media/{}", matched.content_id)
    }
}
