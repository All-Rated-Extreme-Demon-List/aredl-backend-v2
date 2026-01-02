use async_trait::async_trait;
use regex::Regex;

use super::super::model::{Provider, ProviderId, ProviderUsage};

pub struct MedalProvider {
    patterns: Vec<Regex>,
}

impl MedalProvider {
    pub fn new() -> Self {
        Self {
            patterns: vec![
                // https://medal.tv/clips/<id>
                // https://medal.tv/pl/clips/<id>
                Regex::new(
                    r"^https?://medal\.tv(?:/[a-z]{2})?/clips/(?P<id>[A-Za-z0-9_-]+)(?:[/?#].*)?$"
                ).unwrap(),

                // https://medal.tv/games/<game>/clips/<id>
                // https://medal.tv/pl/games/<game>/clips/<id>
                Regex::new(
                    r"^https?://medal\.tv(?:/[a-z]{2})?/games/[A-Za-z0-9_-]+/clips/(?P<id>[A-Za-z0-9_-]+)(?:[/?#].*)?$"
                ).unwrap(),
            ],
        }
    }
}

#[async_trait]
impl Provider for MedalProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Medal
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
        format!("https://medal.tv/clips/{}", content_id)
    }
}
