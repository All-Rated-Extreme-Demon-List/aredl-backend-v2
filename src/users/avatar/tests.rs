use crate::auth::{create_test_token, oauth::OAuthProvider, Permission};
use crate::providers::test_utils::{clear_oauth_env, set_oauth_env};
use crate::test_utils::{assert_error_response, init_test_app};
use crate::users::test_utils::{create_test_user, get_test_user, set_test_user_discord_id};
use actix_web::{http::StatusCode, test};
use httpmock::{Method::GET, MockServer};
use serde_json::json;
use serial_test::serial;

#[actix_web::test]
#[serial]
async fn refresh_updates_target_and_rate_limit_is_respected() {
    let server = MockServer::start_async().await;
    set_oauth_env(OAuthProvider::Discord, &server.base_url());
    std::env::set_var("DISCORD_BOT_TOKEN", "test_bot_token");
    let mock = server.mock_async(|when, then| {
        when.method(GET).path("/api/v10/users/123");
        then.status(200)
            .header("x-ratelimit-remaining", "1")
            .header("x-ratelimit-reset-after", "10")
            .json_body(json!({"avatar": "new-avatar", "avatar_decoration_data": {"asset": "new-decoration"}}));
    }).await;
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, username) = create_test_user(&db, None).await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::UserModify)).await;
    set_test_user_discord_id(&db, user_id, "123").await;
    let token =
        create_test_token(staff_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri(&format!("/users/{username}/avatar/refresh"))
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request(),
    )
    .await;
    assert_eq!(response.status(), StatusCode::OK);
    let user: serde_json::Value = test::read_body_json(response).await;
    assert_eq!(user["id"], json!(user_id));
    assert_eq!(user["discord_avatar"], json!("new-avatar"));
    assert_eq!(user["discord_avatar_decoration"], json!("new-decoration"));
    let stored_user = get_test_user(&db, user_id);
    assert_eq!(stored_user.discord_avatar.as_deref(), Some("new-avatar"));
    assert_eq!(
        stored_user.discord_avatar_decoration.as_deref(),
        Some("new-decoration")
    );

    let user_token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/users/@me/avatar/refresh")
            .insert_header(("Authorization", format!("Bearer {user_token}")))
            .to_request(),
    )
    .await;
    assert!(response.headers().contains_key("Retry-After"));
    assert_error_response!(response, StatusCode::TOO_MANY_REQUESTS, None);
    mock.assert_calls_async(1).await;
    clear_oauth_env(OAuthProvider::Discord);
    std::env::remove_var("DISCORD_BOT_TOKEN");
}

#[actix_web::test]
#[serial]
async fn refresh_fails_for_user_without_discord() {
    let server = MockServer::start_async().await;
    set_oauth_env(OAuthProvider::Discord, &server.base_url());
    std::env::set_var("DISCORD_BOT_TOKEN", "test_bot_token");
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let response = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/users/@me/avatar/refresh")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .to_request(),
    )
    .await;
    assert_error_response!(
        response,
        StatusCode::BAD_REQUEST,
        Some("User has no linked Discord account")
    );
    clear_oauth_env(OAuthProvider::Discord);
    std::env::remove_var("DISCORD_BOT_TOKEN");
}
