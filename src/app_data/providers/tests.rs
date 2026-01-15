#[cfg(test)]
use {
    crate::{
        app_data::providers::model::{Provider, ProviderId},
        providers::{
            context::{GoogleAuthState, ProviderContext, TwitchAuthState},
            init_app_state,
            list::{medal::MedalProvider, youtube::YouTubeProvider},
            test_utils::{
                clear_google_env, clear_twitch_env, mock_google_token_endpoint,
                mock_medal_content_endpoint, mock_twitch_token_endpoint,
                mock_twitch_videos_endpoint, mock_youtube_videos_endpoint, set_google_env,
                set_twitch_env,
            },
        },
    },
    chrono::{DateTime, Utc},
    httpmock::prelude::*,
    serial_test::serial,
    std::sync::Arc,
};

#[tokio::test]
async fn match_and_normalize_all_supported_providers() {
    let providers = init_app_state().await;

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
    let providers = init_app_state().await;

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

#[actix_web::test]
#[serial]
async fn google_auth_fetches_once_then_uses_cache() {
    clear_google_env();

    let server = MockServer::start_async().await;
    set_google_env(&server.base_url());

    let mock = mock_google_token_endpoint(&server, 3600, "token1").await;

    let state = GoogleAuthState::new()
        .await
        .expect("expected GoogleAuthState");

    let t1 = state.get_access_token().await.unwrap();
    let t2 = state.get_access_token().await.unwrap();

    assert_eq!(t1, "token1");
    assert_eq!(t2, "token1");
    assert_eq!(
        mock.calls_async().await,
        1,
        "should only request token once"
    );

    clear_google_env();
}

#[actix_web::test]
#[serial]
async fn google_auth_refreshes_when_immediately_expired() {
    clear_google_env();

    let server = MockServer::start_async().await;
    set_google_env(&server.base_url());

    let mock = mock_google_token_endpoint(&server, 60, "token1").await;

    let state = GoogleAuthState::new()
        .await
        .expect("expected GoogleAuthState");

    let _ = state.get_access_token().await.unwrap();
    let _ = state.get_access_token().await.unwrap();

    assert_eq!(
        mock.calls_async().await,
        2,
        "should refresh due to immediate expiry"
    );

    clear_google_env();
}

#[actix_web::test]
#[serial]
async fn google_auth_refreshes_after_expiry() {
    clear_google_env();

    let server = MockServer::start_async().await;
    set_google_env(&server.base_url());

    let mock = mock_google_token_endpoint(&server, 61, "token1").await;

    let state = GoogleAuthState::new()
        .await
        .expect("expected GoogleAuthState");

    let _ = state.get_access_token().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    let _ = state.get_access_token().await.unwrap();

    assert_eq!(mock.calls_async().await, 2, "should refresh after expiry");

    clear_google_env();
}

#[actix_web::test]
#[serial]
async fn twitch_auth_fetches_once_then_uses_cache() {
    clear_twitch_env();

    let server = MockServer::start_async().await;
    set_twitch_env(&server.base_url());

    let mock = mock_twitch_token_endpoint(&server, 3600, "token1").await;

    let state = TwitchAuthState::new()
        .await
        .expect("expected TwitchAuthState");

    let t1 = state.get_access_token().await.unwrap();
    let t2 = state.get_access_token().await.unwrap();

    assert_eq!(t1, "token1");
    assert_eq!(t2, "token1");
    assert_eq!(
        mock.calls_async().await,
        1,
        "should only request token once"
    );

    clear_twitch_env();
}

#[actix_web::test]
#[serial]
async fn twitch_auth_refreshes_when_immediately_expired() {
    clear_twitch_env();

    let server = MockServer::start_async().await;
    set_twitch_env(&server.base_url());

    let mock = mock_twitch_token_endpoint(&server, 60, "token1").await;

    let state = TwitchAuthState::new()
        .await
        .expect("expected TwitchAuthState");

    let _ = state.get_access_token().await.unwrap();
    let _ = state.get_access_token().await.unwrap();

    assert_eq!(
        mock.calls_async().await,
        2,
        "should refresh due to immediate expiry"
    );

    clear_twitch_env();
}

#[actix_web::test]
#[serial]
async fn twitch_auth_refreshes_after_expiry() {
    clear_twitch_env();

    let server = MockServer::start_async().await;
    set_twitch_env(&server.base_url());

    let mock = mock_twitch_token_endpoint(&server, 61, "token1").await;

    let state = TwitchAuthState::new()
        .await
        .expect("expected TwitchAuthState");

    let _ = state.get_access_token().await.unwrap();
    tokio::time::sleep(std::time::Duration::from_millis(1200)).await;
    let _ = state.get_access_token().await.unwrap();

    assert_eq!(mock.calls_async().await, 2, "should refresh after expiry");

    clear_twitch_env();
}

#[actix_web::test]
#[serial]
async fn youtube_fetch_metadata_returns_published_at() {
    clear_google_env();

    let server = MockServer::start_async().await;
    set_google_env(&server.base_url());
    mock_google_token_endpoint(&server, 3600, "test_access").await;

    let yt_mock =
        mock_youtube_videos_endpoint(&server, "xvFZjo5PgG0", "2009-10-25T06:57:33Z").await;

    let google_auth = GoogleAuthState::new()
        .await
        .expect("Failed to create GoogleAuthState");

    let context = ProviderContext {
        http: reqwest::Client::new(),
        google_auth: Some(Arc::new(google_auth)),
        twitch_auth: None,
    };

    std::env::set_var("YOUTUBE_API_BASE_URL", server.base_url());

    let provider = YouTubeProvider::new();

    let matched = provider
        .parse_url("https://youtube.com/watch?v=xvFZjo5PgG0")
        .unwrap()
        .expect("youtube url should match");

    let meta = provider
        .fetch_metadata(&matched, &context)
        .await
        .expect("fetch_metadata must succeed")
        .expect("metadata must exist");

    assert_eq!(yt_mock.calls_async().await, 1);

    let expected: DateTime<Utc> = "2009-10-25T06:57:33Z".parse().unwrap();
    assert_eq!(meta.published_at, Some(expected));

    std::env::remove_var("YOUTUBE_API_BASE_URL");
    clear_google_env();
}

#[actix_web::test]
async fn medal_fetch_metadata_returns_published_at_from_created_ms() {
    let server = MockServer::start_async().await;

    let created_ms: i64 = 1256453853000;

    let medal_mock = mock_medal_content_endpoint(&server, "jyEnIYev353GxMXDV", created_ms).await;

    let context = ProviderContext {
        http: reqwest::Client::new(),
        google_auth: None,
        twitch_auth: None,
    };

    std::env::set_var("MEDAL_API_BASE_URL", server.base_url());

    let provider = MedalProvider::new();

    let matched = provider
        .parse_url("https://medal.tv/clips/jyEnIYev353GxMXDV")
        .unwrap()
        .expect("medal url should match");

    let meta = provider
        .fetch_metadata(&matched, &context)
        .await
        .expect("fetch_metadata must succeed")
        .expect("metadata must exist");

    assert_eq!(medal_mock.calls_async().await, 1);

    let expected: DateTime<Utc> = "2009-10-25T06:57:33Z".parse().unwrap();
    assert_eq!(meta.published_at, Some(expected));

    std::env::remove_var("MEDAL_API_BASE_URL");
}

#[actix_web::test]
#[serial]
async fn twitch_fetch_metadata_returns_published_at() {
    use crate::providers::list::twitch::TwitchProvider;

    clear_twitch_env();

    let server = MockServer::start_async().await;

    set_twitch_env(&server.base_url());
    let token_mock = mock_twitch_token_endpoint(&server, 3600, "test_access").await;

    std::env::set_var("TWITCH_API_BASE_URL", server.base_url());
    let twitch_mock =
        mock_twitch_videos_endpoint(&server, "987654321", "2009-10-25T06:57:33Z").await;

    let twitch_auth = TwitchAuthState::new()
        .await
        .expect("Failed to create TwitchAuthState");

    let context = ProviderContext {
        http: reqwest::Client::new(),
        google_auth: None,
        twitch_auth: Some(Arc::new(twitch_auth)),
    };

    let provider = TwitchProvider::new();

    let matched = provider
        .parse_url("https://www.twitch.tv/videos/987654321")
        .unwrap()
        .expect("twitch url should match");

    let meta = provider
        .fetch_metadata(&matched, &context)
        .await
        .expect("fetch_metadata must succeed")
        .expect("metadata must exist");

    assert_eq!(
        token_mock.calls_async().await,
        1,
        "token should be fetched once"
    );
    assert_eq!(
        twitch_mock.calls_async().await,
        1,
        "twitch should be called once"
    );

    let expected: DateTime<Utc> = "2009-10-25T06:57:33Z".parse().unwrap();
    assert_eq!(meta.published_at, Some(expected));

    std::env::remove_var("TWITCH_API_BASE_URL");
    clear_twitch_env();
}
