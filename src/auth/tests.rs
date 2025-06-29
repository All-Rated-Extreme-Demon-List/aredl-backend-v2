#[cfg(test)]
use {
    crate::{auth::token, test_utils::init_test_app, users::test_utils::create_test_user},
    actix_web::{http::header, test},
};

#[actix_web::test]
async fn discord_auth_redirects() {
    let (app, _, _, _) = init_test_app().await;

    let req = test::TestRequest::get().uri("/auth/discord").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), actix_web::http::StatusCode::FOUND);
    assert!(resp.headers().contains_key(header::LOCATION));
}

#[actix_web::test]
async fn discord_refresh_returns_token() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, None).await;

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
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.get("access_token").is_some());
}
