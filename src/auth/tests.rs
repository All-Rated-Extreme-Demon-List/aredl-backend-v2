#[cfg(test)]
use {
    crate::{
        auth::{create_test_token, token},
        external_connections::OAuthProvider,
        providers::test_utils::{clear_patreon_env, set_discord_env, set_patreon_env},
        schema::{
            oauth_connected_accounts, oauth_requests,
            users::dsl::{access_valid_after, id as user_id_col, users},
        },
        test_utils::init_test_app,
        users::test_utils::create_test_user,
    },
    actix_web::{
        http::header,
        test::{self, read_body_json},
    },
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    httpmock::{prelude::*, Mock},
    serial_test::serial,
};

#[cfg(test)]
async fn mock_patreon_code_exchange<'a>(
    server: &'a MockServer,
    code: &'a str,
    access_token: &'a str,
) -> Mock<'a> {
    server
        .mock_async(move |when, then| {
            when.method(POST)
                .path("/api/oauth2/token")
                .body_includes("grant_type=authorization_code")
                .body_includes(format!("code={}", code))
                .body_includes("client_id=test_patreon_client_id")
                .body_includes("client_secret=test_patreon_client_secret");

            then.status(200)
                .header("content-type", "application/json")
                .body(format!(
                    r#"{{ "access_token": "{}", "token_type": "Bearer", "expires_in": 3600 }}"#,
                    access_token
                ));
        })
        .await
}

#[cfg(test)]
async fn mock_patreon_identity<'a>(
    server: &'a MockServer,
    access_token: &'a str,
    patreon_id: &'a str,
    full_name: &'a str,
) -> Mock<'a> {
    server
        .mock_async(move |when, then| {
            when.method(GET)
                .path("/api/oauth2/v2/identity")
                .header("authorization", format!("Bearer {}", access_token));

            then.status(200)
                .header("content-type", "application/json")
                .json_body(serde_json::json!({
                    "data": {
                        "id": patreon_id,
                        "type": "user",
                        "attributes": {
                            "full_name": full_name,
                            "vanity": null
                        }
                    }
                }));
        })
        .await
}

#[cfg(test)]
fn patreon_connections_for_user(
    db: &std::sync::Arc<crate::app_data::db::DbAppState>,
    user_id: uuid::Uuid,
) -> Vec<String> {
    oauth_connected_accounts::table
        .filter(oauth_connected_accounts::provider.eq(OAuthProvider::Patreon))
        .filter(oauth_connected_accounts::user_id.eq(user_id))
        .select(oauth_connected_accounts::provider_user_id)
        .load::<String>(&mut db.connection().unwrap())
        .unwrap()
}

#[actix_web::test]
async fn discord_auth_redirects_to_discord() {
    std::env::set_var("AUTH_CALLBACK_ALLOWED_DOMAINS", "example.com");
    std::env::remove_var("AUTH_CALLBACK_ALLOW_LOCALHOST");
    let discord_base_url = "https://test-discord.com";
    set_discord_env(discord_base_url);

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
#[serial]
async fn discord_auth_allows_localhost_callback_by_default() {
    std::env::set_var("AUTH_CALLBACK_ALLOWED_DOMAINS", "example.com");
    std::env::remove_var("AUTH_CALLBACK_ALLOW_LOCALHOST");
    set_discord_env("https://test-discord.com");

    let (app, _, _, _) = init_test_app().await;

    let req = test::TestRequest::get()
        .uri("/auth/discord?callback=http://localhost:3000/login")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::FOUND);
}

#[actix_web::test]
#[serial]
async fn discord_auth_rejects_untrusted_callback() {
    std::env::set_var("AUTH_CALLBACK_ALLOWED_DOMAINS", "example.com");
    std::env::remove_var("AUTH_CALLBACK_ALLOW_LOCALHOST");
    set_discord_env("https://test-discord.com");

    let (app, _, _, _) = init_test_app().await;

    let req = test::TestRequest::get()
        .uri("/auth/discord?callback=https://unauthorized.com")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[actix_web::test]
#[serial]
async fn discord_auth_allows_configured_subdomain_callback() {
    std::env::set_var("AUTH_CALLBACK_ALLOWED_DOMAINS", "example.com");
    std::env::remove_var("AUTH_CALLBACK_ALLOW_LOCALHOST");
    set_discord_env("https://test-discord.com");

    let (app, _, _, _) = init_test_app().await;

    let req = test::TestRequest::get()
        .uri("/auth/discord?callback=https://app.example.com/login")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::FOUND);
}

#[actix_web::test]
async fn patreon_link_requires_authentication() {
    let (app, _, _, _) = init_test_app().await;

    let req = test::TestRequest::post()
        .uri("/auth/patreon/link")
        .set_json(serde_json::json!({}))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::UNAUTHORIZED);
}

#[actix_web::test]
#[serial]
async fn patreon_link_rejects_untrusted_callback() {
    std::env::set_var("AUTH_CALLBACK_ALLOWED_DOMAINS", "example.com");
    std::env::remove_var("AUTH_CALLBACK_ALLOW_LOCALHOST");

    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::post()
        .uri("/auth/patreon/link")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({"callback": "https://unauthorized.com"}))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[actix_web::test]
#[serial]
async fn patreon_link_returns_authorize_url() {
    clear_patreon_env();
    std::env::set_var("AUTH_CALLBACK_ALLOWED_DOMAINS", "example.com");
    std::env::remove_var("AUTH_CALLBACK_ALLOW_LOCALHOST");

    let server = MockServer::start_async().await;
    set_patreon_env(&server.base_url());

    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::post()
        .uri("/auth/patreon/link")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(serde_json::json!({"callback": "https://example.com/patreon/linked"}))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    let authorize_url = body["authorize_url"].as_str().unwrap();

    assert!(authorize_url.starts_with(&format!("{}/oauth2/authorize", server.base_url())));
    assert!(authorize_url.contains("response_type=code"));
    assert!(authorize_url.contains("client_id=test_patreon_client_id"));
    assert!(authorize_url.contains("scope=identity"));
    assert!(authorize_url.contains("state="));

    clear_patreon_env();
}

#[actix_web::test]
async fn refresh_returns_new_token() {
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
        .uri("/auth/refresh")
        .insert_header(("Authorization", format!("Bearer {}", refresh)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body.get("access_token").is_some());
    assert!(body.get("refresh_token").is_none());
}

#[actix_web::test]
async fn refresh_returns_both_tokens_when_about_to_expire() {
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
        .uri("/auth/refresh")
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
async fn refresh_fails_without_token() {
    let (app, _, _, _) = init_test_app().await;

    let req = test::TestRequest::get().uri("/auth/refresh").to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), actix_web::http::StatusCode::BAD_REQUEST);
}

#[actix_web::test]
async fn refresh_fails_with_invalid_token() {
    let (app, _, _, _) = init_test_app().await;

    let req = test::TestRequest::get()
        .uri("/auth/refresh")
        .insert_header(("Authorization", "Bearer invalid_token"))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(!resp.status().is_success());
}

#[actix_web::test]
async fn refresh_fails_with_access_token() {
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
        .uri("/auth/refresh")
        .insert_header(("Authorization", format!("Bearer {}", access_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(!resp.status().is_success());
}

#[actix_web::test]
async fn refresh_fails_with_expired_token() {
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
        .uri("/auth/refresh")
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

    set_discord_env(&server.base_url());

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
    std::env::set_var("AUTH_CALLBACK_ALLOWED_DOMAINS", "example.com");
    std::env::remove_var("AUTH_CALLBACK_ALLOW_LOCALHOST");

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

    set_discord_env(&server.base_url());

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
#[serial]
async fn patreon_callback_links_current_site_user() {
    clear_patreon_env();

    let server = MockServer::start_async().await;
    set_patreon_env(&server.base_url());

    mock_patreon_code_exchange(&server, "abc", "patreon_access").await;
    mock_patreon_identity(&server, "patreon_access", "patreon_123", "Patron One").await;

    let (app, db, _, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;

    diesel::insert_into(oauth_requests::table)
        .values((
            oauth_requests::csrf_state.eq("patreon_state"),
            oauth_requests::pkce_verifier.eq::<Option<String>>(None),
            oauth_requests::callback.eq::<Option<String>>(None),
            oauth_requests::provider.eq(OAuthProvider::Patreon),
            oauth_requests::user_id.eq(Some(user_id)),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = test::TestRequest::get()
        .uri("/auth/patreon/callback?code=abc&state=patreon_state")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    assert_eq!(
        patreon_connections_for_user(&db, user_id),
        vec!["patreon_123"]
    );

    clear_patreon_env();
}

#[actix_web::test]
#[serial]
async fn patreon_callback_transfers_existing_patreon_link() {
    clear_patreon_env();

    let server = MockServer::start_async().await;
    set_patreon_env(&server.base_url());

    mock_patreon_code_exchange(&server, "abc", "patreon_access").await;
    mock_patreon_identity(&server, "patreon_access", "patreon_123", "Patron One").await;

    let (app, db, _, _) = init_test_app().await;
    let (previous_user, _) = create_test_user(&db, None).await;
    let (current_user, _) = create_test_user(&db, None).await;

    diesel::insert_into(oauth_connected_accounts::table)
        .values((
            oauth_connected_accounts::user_id.eq(previous_user),
            oauth_connected_accounts::provider.eq(OAuthProvider::Patreon),
            oauth_connected_accounts::provider_user_id.eq("patreon_123"),
            oauth_connected_accounts::provider_user_name.eq::<Option<String>>(None),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    diesel::insert_into(oauth_requests::table)
        .values((
            oauth_requests::csrf_state.eq("patreon_state"),
            oauth_requests::pkce_verifier.eq::<Option<String>>(None),
            oauth_requests::callback.eq::<Option<String>>(None),
            oauth_requests::provider.eq(OAuthProvider::Patreon),
            oauth_requests::user_id.eq(Some(current_user)),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = test::TestRequest::get()
        .uri("/auth/patreon/callback?code=abc&state=patreon_state")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    assert!(patreon_connections_for_user(&db, previous_user).is_empty());
    assert_eq!(
        patreon_connections_for_user(&db, current_user),
        vec!["patreon_123"]
    );

    clear_patreon_env();
}

#[actix_web::test]
#[serial]
async fn patreon_callback_replaces_current_users_old_patreon_link() {
    clear_patreon_env();

    let server = MockServer::start_async().await;
    set_patreon_env(&server.base_url());

    mock_patreon_code_exchange(&server, "abc", "patreon_access").await;
    mock_patreon_identity(&server, "patreon_access", "patreon_new", "Patron Two").await;

    let (app, db, _, _) = init_test_app().await;
    let (current_user, _) = create_test_user(&db, None).await;

    diesel::insert_into(oauth_connected_accounts::table)
        .values((
            oauth_connected_accounts::user_id.eq(current_user),
            oauth_connected_accounts::provider.eq(OAuthProvider::Patreon),
            oauth_connected_accounts::provider_user_id.eq("patreon_old"),
            oauth_connected_accounts::provider_user_name.eq::<Option<String>>(None),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    diesel::insert_into(oauth_requests::table)
        .values((
            oauth_requests::csrf_state.eq("patreon_state"),
            oauth_requests::pkce_verifier.eq::<Option<String>>(None),
            oauth_requests::callback.eq::<Option<String>>(None),
            oauth_requests::provider.eq(OAuthProvider::Patreon),
            oauth_requests::user_id.eq(Some(current_user)),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = test::TestRequest::get()
        .uri("/auth/patreon/callback?code=abc&state=patreon_state")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success());
    assert_eq!(
        patreon_connections_for_user(&db, current_user),
        vec!["patreon_new"]
    );

    clear_patreon_env();
}

#[actix_web::test]
async fn discord_callback_fails_with_invalid_state() {
    set_discord_env("https://test-discord.com");
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
    set_discord_env("https://test-discord.com");
    let (app, _, _, _) = init_test_app().await;

    let req = test::TestRequest::get()
        .uri("/auth/discord/callback?code=abc&state=nonexistent_state")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert!(!resp.status().is_success());
}
