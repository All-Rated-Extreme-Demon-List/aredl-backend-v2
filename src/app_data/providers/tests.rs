#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{
        app_data::providers::model::ProviderId,
        providers::{init_app_state, VideoProvidersAppState},
    };

    async fn providers() -> Arc<VideoProvidersAppState> {
        init_app_state().await
    }

    #[tokio::test]
    async fn match_and_normalize_all_supported_providers() {
        let providers = providers().await;

        // (input_url, expected_provider, expected_content_id, expected_normalized_url)
        let cases: Vec<(&str, ProviderId, &str, &str)> = vec![
            // YouTube
            (
                "https://youtube.com/watch?v=xvFZjo5PgG0&si=123456&ab_channel=Foo",
                ProviderId::YouTube,
                "xvFZjo5PgG0",
                "https://www.youtube.com/watch?v=xvFZjo5PgG0",
            ),
            (
                "https://youtu.be/xvFZjo5PgG0?si=123456&ab_channel=Foo&t=123",
                ProviderId::YouTube,
                "xvFZjo5PgG0",
                "https://www.youtube.com/watch?v=xvFZjo5PgG0&t=123",
            ),
            (
                "https://youtu.be/xvFZjo5PgG0?t=123&si=123456&ab_channel=Foo",
                ProviderId::YouTube,
                "xvFZjo5PgG0",
                "https://www.youtube.com/watch?v=xvFZjo5PgG0&t=123",
            ),
            (
                "https://www.youtube.com/shorts/xvFZjo5PgG0?si=abc123",
                ProviderId::YouTube,
                "xvFZjo5PgG0",
                "https://www.youtube.com/watch?v=xvFZjo5PgG0",
            ),
            (
                "https://youtube.com/shorts/xvFZjo5PgG0?t=90&si=abc123",
                ProviderId::YouTube,
                "xvFZjo5PgG0",
                "https://www.youtube.com/watch?v=xvFZjo5PgG0&t=90",
            ),
            (
                "https://youtube.com/shorts/xvFZjo5PgG0?si=abc123&t=90",
                ProviderId::YouTube,
                "xvFZjo5PgG0",
                "https://www.youtube.com/watch?v=xvFZjo5PgG0&t=90",
            ),
            (
                "https://www.youtube.com/live/xvFZjo5PgG0?si=abc123",
                ProviderId::YouTube,
                "xvFZjo5PgG0",
                "https://www.youtube.com/watch?v=xvFZjo5PgG0",
            ),
            (
                "https://youtube.com/live/xvFZjo5PgG0?start=123&foo=bar",
                ProviderId::YouTube,
                "xvFZjo5PgG0",
                "https://www.youtube.com/watch?v=xvFZjo5PgG0&t=123",
            ),
            (
                "https://youtube.com/live/xvFZjo5PgG0?foo=bar&t=123",
                ProviderId::YouTube,
                "xvFZjo5PgG0",
                "https://www.youtube.com/watch?v=xvFZjo5PgG0&t=123",
            ),
            (
                "https://m.youtube.com/shorts/xvFZjo5PgG0?t=90",
                ProviderId::YouTube,
                "xvFZjo5PgG0",
                "https://www.youtube.com/watch?v=xvFZjo5PgG0&t=90",
            ),
            // Vimeo
            (
                "https://vimeo.com/123456789",
                ProviderId::Vimeo,
                "123456789",
                "https://vimeo.com/123456789",
            ),
            (
                "https://player.vimeo.com/video/123456789",
                ProviderId::Vimeo,
                "123456789",
                "https://vimeo.com/123456789",
            ),
            // Twitch
            (
                "https://www.twitch.tv/videos/987654321?t=1h2m3s",
                ProviderId::Twitch,
                "987654321",
                "https://www.twitch.tv/videos/987654321?t=1h2m3s",
            ),
            (
                "https://www.twitch.tv/some_channel/video/987654321",
                ProviderId::Twitch,
                "987654321",
                "https://www.twitch.tv/videos/987654321",
            ),
            // BiliBili
            (
                "https://www.bilibili.com/video/BV1xx411c7mD",
                ProviderId::BiliBili,
                "BV1xx411c7mD",
                "https://www.bilibili.com/video/BV1xx411c7mD",
            ),
            // Medal
            (
                "https://medal.tv/clips/abcDEF_-123",
                ProviderId::Medal,
                "abcDEF_-123",
                "https://medal.tv/clips/abcDEF_-123",
            ),
			(
                "https://medal.tv/games/valorant/clips/abcDEF_-123",
                ProviderId::Medal,
                "abcDEF_-123",
                "https://medal.tv/clips/abcDEF_-123",
            ),
            (
                "https://medal.tv/pl/clips/abcDEF_-123",
                ProviderId::Medal,
                "abcDEF_-123",
                "https://medal.tv/clips/abcDEF_-123",
            ),
            (
                "https://medal.tv/pl/games/valorant/clips/abcDEF_-123",
                ProviderId::Medal,
                "abcDEF_-123",
                "https://medal.tv/clips/abcDEF_-123",
            ),
            // Outplayed
            (
                "https://outplayed.tv/media/abcDEF_-123",
                ProviderId::Outplayed,
                "abcDEF_-123",
                "https://outplayed.tv/media/abcDEF_-123",
            ),
            (
                "https://outplayed.tv/valorant/abcDEF_-123",
                ProviderId::Outplayed,
                "abcDEF_-123",
                "https://outplayed.tv/media/abcDEF_-123",
            ),
            // Google Drive
            (
				"https://drive.google.com/file/d/1AbCdEfGhIj/view?usp=sharing",
				ProviderId::GoogleDrive,
				"1AbCdEfGhIj",
				"https://drive.google.com/file/d/1AbCdEfGhIj",
			),
			(
				"https://drive.google.com/u/0/file/d/1AbCdEfGhIj/view?usp=sharing",
				ProviderId::GoogleDrive,
				"1AbCdEfGhIj",
				"https://drive.google.com/file/d/1AbCdEfGhIj",
			),
			(
				"https://drive.google.com/drive/folders/1NH3sbowKcOP5KOTAfJ4mhaUleUnZgd0m?usp=sharing",
				ProviderId::GoogleDrive,
				"1NH3sbowKcOP5KOTAfJ4mhaUleUnZgd0m",
				"https://drive.google.com/drive/folders/1NH3sbowKcOP5KOTAfJ4mhaUleUnZgd0m",
			),
			(
				"https://drive.google.com/drive/u/0/folders/1NH3sbowKcOP5KOTAfJ4mhaUleUnZgd0m?usp=sharing",
				ProviderId::GoogleDrive,
				"1NH3sbowKcOP5KOTAfJ4mhaUleUnZgd0m",
				"https://drive.google.com/drive/folders/1NH3sbowKcOP5KOTAfJ4mhaUleUnZgd0m",
			),
			(
				"https://drive.google.com/open?id=1AbCdEfGhIj",
				ProviderId::GoogleDrive,
				"1AbCdEfGhIj",
				"https://drive.google.com/file/d/1AbCdEfGhIj",
			),
			(
				"https://drive.google.com/uc?id=1AbCdEfGhIj",
				ProviderId::GoogleDrive,
				"1AbCdEfGhIj",
				"https://drive.google.com/file/d/1AbCdEfGhIj",
			),
            // Mega
            (
                "https://mega.nz/file/AbCDeF#GhIjKlMn",
                ProviderId::Mega,
                "AbCDeF",
                "https://mega.nz/file/AbCDeF#GhIjKlMn",
            ),
        ];

        for (input, provider, id, expected_norm) in cases {
            let validated_input = providers.validate_is_url(input).unwrap_or_else(|e| {
                panic!(
                    "Expected valid URL for {input}, got error: {}",
                    e.error_message
                )
            });
            let m = providers.parse_url(&validated_input).unwrap_or_else(|e| {
                panic!("Expected match for {input}, got error: {}", e.error_message)
            });

            assert_eq!(m.provider, provider, "provider mismatch for {input}");
            assert_eq!(m.content_id, id, "content_id mismatch for {input}");
            assert_eq!(
                m.normalized_url, expected_norm,
                "normalized_url mismatch for {input}"
            );
        }
    }

    #[tokio::test]
    async fn reject_unsupported_or_malformed_urls() {
        let providers = providers().await;

        let invalid_urls: Vec<&str> = vec![
            "not a url",
            "xxx https://youtube.com/watch?v=xvFZjo5PgG0",
            "https://youtube.com/watch?v=xvFZjo5PgG0 yyy",
        ];

        let nomatch_urls: Vec<&str> = vec![
            "https://example.com/watch?v=abcdef",
            "https://www.twitch.tv/directory",
            "https://medal.tv/u/someuser",
            "https://medal.tv/share/something",
            "https://drive.google.com/folders/1NH3sbowKcOP5KOTAfJ4mhaUleUnZgd0m?usp=sharing",
            "https://drive.google.com/folders/",
            "https://drive.google.com/drive/u/0/my-drive",
            "https://drive.google.com/drive/u/0/shared-with-me",
        ];

        for url in invalid_urls {
            let validated = providers.validate_is_url(url);
            assert!(
                validated.is_err(),
                "Expected invalid URL error for {url}, but got valid"
            );
        }

        for url in nomatch_urls {
            let validated = providers.validate_is_url(url).unwrap_or_else(|e| {
                panic!(
                    "Expected valid URL for {url}, got error: {}",
                    e.error_message
                )
            });
            let matched = providers.parse_url(&validated);
            assert!(
                matched.is_err(),
                "Expected no match error for {url}, but got a match"
            );
        }
    }
}
