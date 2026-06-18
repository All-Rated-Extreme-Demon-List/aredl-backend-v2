#[cfg(test)]
use httpmock::{prelude::*, Mock};

#[cfg(test)]
use crate::providers::context::backend_oauth::oauth_token_aad;

#[cfg(test)]
use {
    crate::{
        app_data::{db::DbAppState, providers::context::decrypt_db_token_value},
        auth::oauth::OAuthProvider,
        schema::oauth_tokens,
    },
    chrono::{DateTime, Utc},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
};

#[cfg(test)]
fn set_token_encryption_env() {
    std::env::set_var(
        "OAUTH_TOKEN_ENCRYPTION_KEY",
        "l/ai+o6bpEWvvdzYuiIHbbN5TeQo8pMaqbZ3u1bvEa4=",
    );
}

#[cfg(test)]
pub fn set_discord_env(server_base: &str) {
    std::env::set_var(
        "DISCORD_OAUTH_CLIENT_CONFIG",
        format!(
            r#"{{
                    "client_id": "test_discord_client_id",
                    "client_secret": "test_discord_client_secret",
                    "issuer_uri": "{0}",
                    "authorize_uri": "{0}/oauth2/authorize",
                    "token_uri": "{0}/api/oauth2/token",
                    "api_base_uri": "{0}",
                    "redirect_uri": "https://example.com/discord/callback",
                    "scopes": ["identify"],
                    "use_pkce": true,
                    "use_openid_scope": true,
                    "auth_type": "request_body"
                }}"#,
            server_base
        ),
    );
}

#[cfg(test)]
pub fn clear_discord_env() {
    std::env::remove_var("DISCORD_OAUTH_CLIENT_CONFIG");
}

#[cfg(test)]
pub fn set_google_env(server_base: &str) {
    set_token_encryption_env();
    std::env::set_var(
        "GOOGLE_OAUTH_CLIENT_CONFIG",
        format!(
            r#"{{
                    "client_id": "test_client_id",
                    "client_secret": "test_client_secret",
                    "token_uri": "{0}/token",
                    "api_base_uri": "{0}",
                    "issuer_uri": "{0}",
                    "authorize_uri": "{0}/oauth2/authorize",
                    "redirect_uri": "https://example.com/google/callback"
                }}"#,
            server_base
        ),
    );
}

#[cfg(test)]
pub fn clear_google_env() {
    std::env::remove_var("GOOGLE_OAUTH_CLIENT_CONFIG");
    std::env::remove_var("OAUTH_TOKEN_ENCRYPTION_KEY");
}

#[cfg(test)]
pub async fn mock_google_token_endpoint<'a>(
    server: &'a MockServer,
    expires_in: u64,
    access_token: &str,
) -> Mock<'a> {
    let access_token = access_token.to_string();

    server
        .mock_async(move |when, then| {
            when.method(POST)
                .path("/token")
                .body_includes("grant_type=refresh_token")
                .body_includes("client_id=test_client_id")
                .body_includes("client_secret=test_client_secret");

            then.status(200)
                .header("content-type", "application/json")
                .body(format!(
                    r#"{{"access_token":"{}","expires_in":{}}}"#,
                    access_token, expires_in
                ));
        })
        .await
}

#[cfg(test)]
pub async fn mock_google_token_refresh_endpoint<'a>(
    server: &'a MockServer,
    expires_in: u64,
    access_token: &str,
    request_refresh_token: &str,
    response_refresh_token: Option<&str>,
) -> Mock<'a> {
    let access_token = access_token.to_string();
    let request_refresh_token = request_refresh_token.to_string();
    let response_refresh_token = response_refresh_token.map(str::to_string);

    server
        .mock_async(move |when, then| {
            when.method(POST)
                .path("/token")
                .body_includes("grant_type=refresh_token")
                .body_includes(format!("refresh_token={}", request_refresh_token))
                .body_includes("client_id=test_client_id")
                .body_includes("client_secret=test_client_secret");

            let refresh_field = response_refresh_token
                .as_ref()
                .map(|refresh_token| format!(r#","refresh_token":"{}""#, refresh_token))
                .unwrap_or_default();

            then.status(200)
                .header("content-type", "application/json")
                .body(format!(
                    r#"{{"access_token":"{}","expires_in":{}{}}}"#,
                    access_token, expires_in, refresh_field
                ));
        })
        .await
}

#[cfg(test)]
pub fn seed_google_token(db: &DbAppState, refresh_token: &str) {
    seed_oauth_token(db, OAuthProvider::Google, Some(refresh_token));
}

#[cfg(test)]
pub fn stored_google_refresh_token(db: &DbAppState) -> String {
    stored_oauth_refresh_token(db, OAuthProvider::Google)
}

#[cfg(test)]
pub fn raw_stored_google_refresh_token(db: &DbAppState) -> String {
    raw_stored_oauth_refresh_token(db, OAuthProvider::Google)
}

#[cfg(test)]
pub fn set_patreon_env(server_base: &str) {
    set_token_encryption_env();
    std::env::set_var(
        "PATREON_OAUTH_CLIENT_CONFIG",
        format!(
            r#"{{
                    "client_id": "test_patreon_client_id",
                    "client_secret": "test_patreon_client_secret",
                    "issuer_uri": "{0}/oauth2/authorize",
                    "authorize_uri": "{0}/oauth2/authorize",
                    "token_uri": "{0}/api/oauth2/token",
                    "api_base_uri": "{0}",
                    "redirect_uri": "https://example.com/patreon/callback",
                    "scopes": ["identity"],
                    "use_pkce": false,
                    "use_openid_scope": false,
                    "auth_type": "request_body"
                }}"#,
            server_base
        ),
    );
}

#[cfg(test)]
pub fn clear_patreon_env() {
    std::env::remove_var("PATREON_OAUTH_CLIENT_CONFIG");
    std::env::remove_var("OAUTH_TOKEN_ENCRYPTION_KEY");
}

#[cfg(test)]
pub async fn mock_patreon_token_endpoint<'a>(
    server: &'a MockServer,
    expires_in: u64,
    access_token: &str,
    request_refresh_token: &str,
    response_refresh_token: &str,
) -> Mock<'a> {
    let access_token = access_token.to_string();
    let request_refresh_token = request_refresh_token.to_string();
    let response_refresh_token = response_refresh_token.to_string();

    server
        .mock_async(move |when, then| {
            when.method(POST)
                .path("/api/oauth2/token")
                .body_includes("grant_type=refresh_token")
                .body_includes(format!("refresh_token={}", request_refresh_token))
                .body_includes("client_id=test_patreon_client_id")
                .body_includes("client_secret=test_patreon_client_secret");

            then.status(200)
                .header("content-type", "application/json")
                .body(format!(
                    r#"{{ "access_token": "{}", "refresh_token": "{}", "expires_in": {} }}"#,
                    access_token, response_refresh_token, expires_in
                ));
        })
        .await
}

#[cfg(test)]
pub fn seed_patreon_token(db: &DbAppState, refresh_token: &str) {
    seed_oauth_token(db, OAuthProvider::Patreon, Some(refresh_token));
}

#[cfg(test)]
fn seed_oauth_token(db: &DbAppState, provider: OAuthProvider, refresh_token: Option<&str>) {
    diesel::insert_into(oauth_tokens::table)
        .values((
            oauth_tokens::provider.eq(provider),
            oauth_tokens::access_token.eq::<Option<String>>(None),
            oauth_tokens::refresh_token.eq(refresh_token.map(str::to_string)),
            oauth_tokens::expires_at.eq::<Option<DateTime<Utc>>>(None),
        ))
        .on_conflict(oauth_tokens::provider)
        .do_update()
        .set((
            oauth_tokens::access_token.eq::<Option<String>>(None),
            oauth_tokens::refresh_token.eq(refresh_token.map(str::to_string)),
            oauth_tokens::expires_at.eq::<Option<DateTime<Utc>>>(None),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();
}

#[cfg(test)]
pub fn stored_patreon_refresh_token(db: &DbAppState) -> String {
    stored_oauth_refresh_token(db, OAuthProvider::Patreon)
}

#[cfg(test)]
fn stored_oauth_refresh_token(db: &DbAppState, provider: OAuthProvider) -> String {
    let refresh_token = raw_stored_oauth_refresh_token(db, provider);

    decrypt_db_token_value(&refresh_token, &oauth_token_aad(provider, "refresh_token")).unwrap()
}

#[cfg(test)]
fn raw_stored_oauth_refresh_token(db: &DbAppState, provider: OAuthProvider) -> String {
    oauth_tokens::table
        .filter(oauth_tokens::provider.eq(provider))
        .select(oauth_tokens::refresh_token)
        .first::<Option<String>>(&mut db.connection().unwrap())
        .unwrap()
        .expect("expected refresh token")
}

#[cfg(test)]
pub fn set_twitch_env(server_base: &str) {
    set_token_encryption_env();
    std::env::set_var(
        "TWITCH_OAUTH_CLIENT_CONFIG",
        format!(
            r#"{{
                    "client_id": "test_twitch_client_id",
                    "client_secret": "test_twitch_client_secret",
                    "token_uri": "{0}/oauth2/token",
                    "api_base_uri": "{0}",
                    "issuer_uri": "{0}",
                    "authorize_uri": "{0}/oauth2/authorize",
                    "redirect_uri": "https://example.com/twitch/callback"
                }}"#,
            server_base
        ),
    );
}

#[cfg(test)]
pub fn clear_twitch_env() {
    std::env::remove_var("TWITCH_OAUTH_CLIENT_CONFIG");
    std::env::remove_var("OAUTH_TOKEN_ENCRYPTION_KEY");
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
                .path("/youtube/v3/videos")
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
