#[cfg(test)]
use httpmock::{prelude::*, Mock};

#[cfg(test)]
pub fn set_google_env(server_base: &str) {
    std::env::set_var(
        "GOOGLE_OAUTH_CLIENT",
        format!(
            r#"{{
                    "web": {{
                        "client_id": "test_client_id",
                        "client_secret": "test_client_secret",
                        "token_uri": "{}/token"
                    }}
                }}"#,
            server_base
        ),
    );

    std::env::set_var(
        "GOOGLE_OAUTH_REFRESH",
        r#"{ "refresh_token": "test_refresh" }"#,
    );
}

#[cfg(test)]
pub fn clear_google_env() {
    std::env::remove_var("GOOGLE_OAUTH_CLIENT");
    std::env::remove_var("GOOGLE_OAUTH_REFRESH");
}

#[cfg(test)]
pub async fn mock_google_token_endpoint<'a>(
    server: &'a MockServer,
    expires_in: u64,
    access_token: &str,
) -> Mock<'a> {
    server
        .mock_async(move |when, then| {
            when.method(POST)
                .path("/token")
                .body_includes("grant_type=refresh_token")
                .body_includes("refresh_token=test_refresh")
                .body_includes("client_id=test_client_id")
                .body_includes("client_secret=test_client_secret");

            then.status(200)
                .header("content-type", "application/json")
                .body(format!(
                    r#"{{ "access_token": "{}", "expires_in": {} }}"#,
                    access_token, expires_in
                ));
        })
        .await
}

#[cfg(test)]
pub fn set_twitch_env(server_base: &str) {
    std::env::set_var("TWITCH_OAUTH_CLIENT_ID", "test_twitch_client_id");
    std::env::set_var("TWITCH_OAUTH_CLIENT_SECRET", "test_twitch_client_secret");
    std::env::set_var(
        "TWITCH_OAUTH_TOKEN_URI",
        format!("{}/oauth2/token", server_base),
    );
}

#[cfg(test)]
pub fn clear_twitch_env() {
    std::env::remove_var("TWITCH_OAUTH_CLIENT_ID");
    std::env::remove_var("TWITCH_OAUTH_CLIENT_SECRET");
    std::env::remove_var("TWITCH_OAUTH_TOKEN_URI");
}

#[cfg(test)]
pub async fn mock_twitch_token_endpoint<'a>(
    server: &'a MockServer,
    expires_in: u64,
    access_token: &str,
) -> Mock<'a> {
    let access_token = access_token.to_string();

    server
        .mock_async(move |when, then| {
            when.method(POST)
                .path("/oauth2/token")
                .body_includes("grant_type=client_credentials")
                .body_includes("client_id=test_twitch_client_id")
                .body_includes("client_secret=test_twitch_client_secret");

            then.status(200)
                .header("content-type", "application/json")
                .body(format!(
                    r#"{{"access_token":"{}","expires_in":{},"token_type":"bearer"}}"#,
                    access_token, expires_in
                ));
        })
        .await
}

#[cfg(test)]
pub async fn mock_youtube_videos_endpoint<'a>(
    server: &'a MockServer,
    video_id: &str,
    published_at: &str,
) -> Mock<'a> {
    let video_id = video_id.to_string();
    let published_at = published_at.to_string();

    server
        .mock_async(move |when, then| {
            when.method(GET)
                .path("/videos")
                .query_param("part", "snippet")
                .query_param("id", &video_id)
                .header_exists("Authorization");

            then.status(200)
                .header("content-type", "application/json")
                .body(format!(
                    r#"{{"items":[{{"snippet":{{"publishedAt":"{}"}}}}]}}"#,
                    published_at
                ));
        })
        .await
}

#[cfg(test)]
pub async fn mock_medal_content_endpoint<'a>(
    server: &'a MockServer,
    clip_id: &str,
    created_ms: i64,
) -> Mock<'a> {
    let clip_id = clip_id.to_string();

    server
        .mock_async(move |when, then| {
            when.method(GET).path(format!("/content/{}", clip_id));

            then.status(200)
                .header("content-type", "application/json")
                .body(format!(r#"{{"created":{}}}"#, created_ms));
        })
        .await
}

#[cfg(test)]
pub async fn mock_twitch_videos_endpoint<'a>(
    server: &'a MockServer,
    video_id: &str,
    published_at: &str,
) -> Mock<'a> {
    let video_id = video_id.to_string();
    let published_at = published_at.to_string();

    server
        .mock_async(move |when, then| {
            when.method(GET)
                .path("/videos")
                .query_param("id", &video_id)
                .header("Client-Id", "test_twitch_client_id")
                .header_exists("Authorization");

            then.status(200)
                .header("content-type", "application/json")
                .body(format!(
                    r#"{{"data":[{{"id":"{id}","published_at":"{pa}"}}]}}"#,
                    id = video_id,
                    pa = published_at
                ));
        })
        .await
}
