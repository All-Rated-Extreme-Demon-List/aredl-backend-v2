use async_trait::async_trait;
use regex::Regex;

use super::super::model::{Provider, ProviderId, ProviderUsage};

pub struct TwitchProvider {
    patterns: Vec<Regex>,
}

impl TwitchProvider {
    pub fn new() -> Self {
        Self {
             patterns: vec![
                // https://www.twitch.tv/videos/<id>[?...][t=...]
                Regex::new(
                    r"^https?://(?:www\.)?twitch\.tv/videos/(?P<id>\d+)(?:\?(?:(?:[^#]*?&)?t=(?P<ts>[^&#]+)[^#]*)?[^#]*)?(?:[&#].*)?$"
                ).unwrap(),
                // https://www.twitch.tv/<channel>/video/<id>[?...][t=...]
                Regex::new(
                    r"^https?://(?:www\.)?twitch\.tv/(?P<other>[A-Za-z0-9_]+)/video/(?P<id>\d+)(?:\?(?:(?:[^#]*?&)?t=(?P<ts>[^&#]+)[^#]*)?[^#]*)?(?:[&#].*)?$"
                ).unwrap(),
                // https://www.twitch.tv/<channel>/v/<id>[?...][t=...]
                Regex::new(
                    r"^https?://(?:www\.)?twitch\.tv/(?P<other>[A-Za-z0-9_]+)/v/(?P<id>\d+)(?:\?(?:(?:[^#]*?&)?t=(?P<ts>[^&#]+)[^#]*)?[^#]*)?(?:[&#].*)?$"
                ).unwrap(),
                // https://player.twitch.tv/?video=v<id>&time=...
                Regex::new(
                    r"^https?://player\.twitch\.tv/\?(?:[^#&]*&)*video=v(?P<id>\d+)(?:&(?:[^#&]*&)*time=(?P<ts>[^&#]+))?(?:[&#].*)?$"
                ).unwrap(),
            ],
        }
    }
}

#[async_trait]
impl Provider for TwitchProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Twitch
    }

    fn usage(&self) -> ProviderUsage {
        ProviderUsage::Both
    }

    fn patterns(&self) -> &[Regex] {
        &self.patterns
    }

    fn normalize_url(
        &self,
        _raw_url: &str,
        content_id: &str,
        timestamp: Option<&str>,
        _channel_id: Option<&str>,
    ) -> String {
        match timestamp {
            Some(t) if !t.is_empty() => {
                format!("https://www.twitch.tv/videos/{}?t={}", content_id, t)
            }
            _ => format!("https://www.twitch.tv/videos/{}", content_id),
        }
    }
}
