#[cfg(test)]
use {
    crate::{
        auth::{create_test_token, token},
        schema::{
            oauth_requests,
            users::dsl::{access_valid_after, id as user_id_col, users},
        },
        test_utils::init_test_app,
        users::test_utils::create_test_user,
    },
    actix_web::{http::header, test::{self, read_body_json}},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    httpmock::prelude::*,
    serial_test::serial,
};

#[actix_web::test]
async fn discord_auth_redirects_to_discord() {
    let discord_base_url = "https://test-discord.com";
    std::env::set_var("DISCORD_BASE_URL", discord_base_url);

    let (app, _, _, _) = init_test_app().await;

    let req = test::TestRequest::get().uri("/auth/discord").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::FOUND);

    let location_header = resp
        .headers()
        .get(header::LOCATION)
        .unwrap()
        .to_str()
        .unwrap();

    assert!(location_header.starts_with(&format!("{}/oauth2/authorize", discord_base_url)));
    assert!(location_header.contains("response_type=code"));
    assert!(location_header.contains("client_id="));
    assert!(location_header.contains("state="));
    assert!(location_header.contains("code_challenge="));
    assert!(location_header.contains("code_challenge_method=S256"));
    assert!(location_header.contains("redirect_uri="));
    assert!(location_header.contains("identify"));
}

#[actix_web::test]
async fn discord_refresh_returns_new_token() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;

    let (refresh, _) = token::create_token(
        token::UserClaims {
            user_id,
            is_api_key: false,
        },
        &auth.jwt_encoding_key,
        chrono::Duration::weeks(2),
        "refresh",
    )
    .unwrap();

    let req = test::TestRequest::get()
        .uri("/auth/discord/refresh")
        .insert_header(("Authorization", format!("Bearer {}", refresh)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body.get("access_token").is_some());
    assert!(body.get("refresh_token").is_none());
}

#[actix_web::test]
async fn discord_refresh_returns_both_tokens_when_about_to_expire() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;

    let (refresh, _) = token::create_token(
        token::UserClaims {
            user_id,
            is_api_key: false,
        },
        &auth.jwt_encoding_key,
        chrono::Duration::minutes(5),
        "refresh",
    )
    .unwrap();

    let req = test::TestRequest::get()
        .uri("/auth/discord/refresh")
        .insert_header(("Authorization", format!("Bearer {}", refresh)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = read_body_json(resp).await;

    assert!(body.get("access_token").is_some());
    assert!(body.get("refresh_token").is_some());

    let access_token = body["access_token"].as_str().unwrap();
    let new_refresh_token = body["refresh_token"].as_str().unwrap();

    let access_claims =
        token::decode_token(access_token, &auth.jwt_decoding_key, &["access"]).unwrap();
    let access_user_claims = token::decode_user_claims(&access_claims).unwrap();
    assert_eq!(access_user_claims.user_id, user_id);
    assert!(!access_user_claims.is_api_key);

    let refresh_claims =
        token::decode_token(new_refresh_token, &auth.jwt_decoding_key, &["refresh"]).unwrap();
    let refresh_user_claims = token::decode_user_claims(&refresh_claims).unwrap();
    assert_eq!(refresh_user_claims.user_id, user_id);
    assert!(!refresh_user_claims.is_api_key);
}

#[actix_web::test]
async fn discord_refresh_fails_without_token() {
    let (app, _, _, _) = init_test_app().await;

    let req = test::TestRequest::get()
        .uri("/auth/discord/refresh")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn discord_refresh_fails_with_invalid_token() {
    let (app, _, _, _) = init_test_app().await;

    let req = test::TestRequest::get()
        .uri("/auth/discord/refresh")
        .insert_header(("Authorization", "Bearer invalid_token"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(!resp.status().is_success());
}

#[actix_web::test]
async fn discord_refresh_fails_with_access_token() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;

    let (access_token, _) = token::create_token(
        token::UserClaims {
            user_id,
            is_api_key: false,
        },
        &auth.jwt_encoding_key,
        chrono::Duration::minutes(30),
        "access",
    )
    .unwrap();

    let req = test::TestRequest::get()
        .uri("/auth/discord/refresh")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(!resp.status().is_success());
}

#[actix_web::test]
async fn discord_refresh_fails_with_expired_token() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;

    let (expired_token, _) = token::create_token(
        token::UserClaims {
            user_id,
            is_api_key: false,
        },
        &auth.jwt_encoding_key,
        chrono::Duration::minutes(-5),
        "refresh",
    )
    .unwrap();

    let req = test::TestRequest::get()
        .uri("/auth/discord/refresh")
        .insert_header(("Authorization", format!("Bearer {}", expired_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(!resp.status().is_success());
}

#[actix_web::test]
async fn create_api_key_generates_token() {
    std::env::set_var("DISCORD_SKIP_DISCOVERY", "1");
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::post()
        .uri("/auth/api-key?lifetime_minutes=10")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    let api_key = body["api_key"].as_str().unwrap();

    let claims = token::decode_token(api_key, &auth.jwt_decoding_key, &["access"]).unwrap();
    let user_claims = token::decode_user_claims(&claims).unwrap();
    assert_eq!(user_claims.user_id, user_id);
    assert!(user_claims.is_api_key);
}

#[actix_web::test]
async fn logout_all_updates_timestamp() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let before: chrono::DateTime<chrono::Utc> = users
        .filter(user_id_col.eq(user_id))
        .select(access_valid_after)
        .first(&mut db.connection().unwrap())
        .unwrap();

    let req = test::TestRequest::post()
        .uri("/auth/logout-all")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let after: chrono::DateTime<chrono::Utc> = users
        .filter(user_id_col.eq(user_id))
        .select(access_valid_after)
        .first(&mut db.connection().unwrap())
        .unwrap();

    assert!(after > before);
}

#[actix_web::test]
#[serial]
async fn discord_callback_returns_auth() {
    let server = MockServer::start_async().await;

    server
        .mock_async(|when, then| {
            when.method(POST).path("/api/oauth2/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                "access_token":"dummy_access",
                "token_type":"Bearer",
                "expires_in":3600,
                "scope":"identify"
            }"#,
                );
        })
        .await;

    server
        .mock_async(|when, then| {
            when.method(GET).path("/api/users/@me");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(serde_json::json!({
                    "id": "123",
                    "username": "tester",
                    "global_name": "tester",
                    "avatar": null,
                    "banner": null,
                    "accent_color": null
                }));
        })
        .await;

    std::env::set_var("DISCORD_BASE_URL", server.base_url());

    let (app, db, _, _) = init_test_app().await;

    diesel::insert_into(oauth_requests::table)
        .values((
            oauth_requests::csrf_state.eq("state"),
            oauth_requests::pkce_verifier.eq("verifier"),
            oauth_requests::callback.eq::<Option<String>>(None),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = test::TestRequest::get()
        .uri("/auth/discord/callback?code=abc&state=state")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());

    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body.get("access_token").is_some());
    assert!(body.get("refresh_token").is_some());
    assert_eq!(body["discord_id"], "123");
    assert_eq!(body["username"], "tester");
}

#[actix_web::test]
#[serial]
async fn discord_callback_with_callback_url_redirects() {
    let server = MockServer::start_async().await;

    server
        .mock_async(|when, then| {
            when.method(POST).path("/api/oauth2/token");
            then.status(200)
                .header("content-type", "application/json")
                .body(
                    r#"{
                "access_token":"dummy_access",
                "token_type":"Bearer",
                "expires_in":3600,
                "scope":"identify"
            }"#,
                );
        })
        .await;

    server
        .mock_async(|when, then| {
            when.method(GET).path("/api/users/@me");
            then.status(200)
                .header("content-type", "application/json")
                .json_body(serde_json::json!({
                    "id": "456",
                    "username": "callback_tester",
                    "global_name": "Callback Tester",
                    "avatar": null,
                    "banner": null,
                    "accent_color": null
                }));
        })
        .await;

    std::env::set_var("DISCORD_BASE_URL", server.base_url());

    let (app, db, _, _) = init_test_app().await;

    diesel::insert_into(oauth_requests::table)
        .values((
            oauth_requests::csrf_state.eq("callback_state"),
            oauth_requests::pkce_verifier.eq("callback_verifier"),
            oauth_requests::callback.eq(Some("https://example.com/auth/success".to_string())),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = test::TestRequest::get()
        .uri("/auth/discord/callback?code=xyz&state=callback_state")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::FOUND);

    let location_header = resp
        .headers()
        .get(header::LOCATION)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(location_header.starts_with("https://example.com/auth/success?token="));
}

#[actix_web::test]
async fn discord_callback_fails_with_invalid_state() {
    let (app, db, _, _) = init_test_app().await;

    diesel::insert_into(oauth_requests::table)
        .values((
            oauth_requests::csrf_state.eq("valid_state"),
            oauth_requests::pkce_verifier.eq("verifier"),
            oauth_requests::callback.eq::<Option<String>>(None),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = test::TestRequest::get()
        .uri("/auth/discord/callback?code=abc&state=invalid_state")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(!resp.status().is_success());
}

#[actix_web::test]
async fn discord_callback_fails_without_oauth_request() {
    let (app, _, _, _) = init_test_app().await;

    let req = test::TestRequest::get()
        .uri("/auth/discord/callback?code=abc&state=nonexistent_state")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(!resp.status().is_success());
}
