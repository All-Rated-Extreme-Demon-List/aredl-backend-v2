use async_trait::async_trait;
use regex::Regex;
use url::Url;

use crate::providers::model::ProviderMatch;

use super::super::model::{Provider, ProviderId, ProviderUsage};

pub struct MegaProvider;

#[async_trait]
impl Provider for MegaProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Mega
    }

    fn usage(&self) -> ProviderUsage {
        ProviderUsage::RawFootage
    }

    fn hosts(&self) -> &'static [&'static str] {
        &["mega.nz", "www.mega.nz"]
    }

    fn match_url(&self, url: &Url) -> Option<ProviderMatch> {
        let path = url.path().trim_matches('/');

        let (content_id, key): (String, Option<String>) = if path.starts_with("file/") {
            // /file/<id>#<key>
            let id = path.strip_prefix("file/")?;
            let fragment = url.fragment()?;
            (id.to_string(), Some(fragment.to_string()))
        } else if path.starts_with("!") {
            // /#!<id>!<key>
            let mut parts = path.trim_start_matches('!').split('!');
            let id = parts.next()?;
            let key = parts.next()?;
            (id.to_string(), Some(key.to_string()))
        } else {
            return None;
        };

        if !Regex::new(r"^[A-Za-z0-9_-]{1,256}$")
            .unwrap()
            .is_match(&content_id)
        {
            return None;
        }

        if let Some(ref key) = key {
            if !Regex::new(r"^[A-Za-z0-9_-]{1,256}$").unwrap().is_match(key) {
                return None;
            }
        }

        Some(ProviderMatch {
            provider: ProviderId::Mega,
            content_id,
            timestamp: None,
            other_id: key,
        })
    }

    fn normalize_url(&self, _raw_url: &Url, matched: &ProviderMatch) -> String {
        match matched.other_id.as_deref() {
            Some(key) => format!("https://mega.nz/file/{}#{}", matched.content_id, key),
            None => format!("https://mega.nz/file/{}", matched.content_id),
        }
    }
}
