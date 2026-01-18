use async_trait::async_trait;
use regex::Regex;
use url::Url;

use crate::providers::model::ProviderMatch;

use super::super::model::{Provider, ProviderId, ProviderUsage};

pub struct VimeoProvider;

#[async_trait]
impl Provider for VimeoProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Vimeo
    }

    fn usage(&self) -> ProviderUsage {
        ProviderUsage::CompletionVideo
    }

    fn hosts(&self) -> &'static [&'static str] {
        &["vimeo.com", "www.vimeo.com", "player.vimeo.com"]
    }

    fn match_url(&self, url: &Url) -> Option<ProviderMatch> {
        let host = url.host_str()?;
        let path = url.path().trim_matches('/');

        let content_id = if host == "player.vimeo.com" {
            // /video/<id>
            let mut it = path.split('/');
            match (it.next(), it.next()) {
                (Some("video"), Some(id)) => Some(id),
                _ => None,
            }
        } else {
            // /<id>
            path.split('/').next()
        }?;

        if !Regex::new(r"^[0-9]{1,20}$").unwrap().is_match(content_id) {
            return None;
        }

        Some(ProviderMatch {
            provider: ProviderId::Vimeo,
            content_id: content_id.to_string(),
            timestamp: None,
            other_id: None,
        })
    }

    fn normalize_url(&self, _raw_url: &Url, matched: &ProviderMatch) -> String {
        format!("https://vimeo.com/{}", matched.content_id)
    }
}
