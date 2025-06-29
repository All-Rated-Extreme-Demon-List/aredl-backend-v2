#[cfg(test)]
use {
    crate::arepl::levels::test_utils::create_test_level,
    crate::auth::create_test_token,
    crate::schema::{arepl::levels, users},
    crate::{test_utils::*, users::test_utils::create_test_user},
    actix_web::test,
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    httpmock::prelude::*,
    serde_json::json,
};

#[actix_web::test]
async fn sync_pemonlist() {
    let server = MockServer::start_async().await;
    let response_body = json!({
        "records": [
            {
                "formatted_time": "00:00:05.100",
                "level": {"level_id": 12345},
                "mobile": false,
                "video_id": "abcd"
            }
        ]
    });

    server
        .mock_async(|when, then| {
            when.method(GET)
                .path(format!("/api/player/{}", "550348841396994048"));
            then.status(200)
                .header("content-type", "application/json")
                .json_body(response_body.clone());
        })
        .await;

    std::env::set_var(
        "PEMONLIST_API_URL",
        format!("{}/api/player", server.base_url()),
    );

    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, None).await;

    diesel::update(users::table.filter(users::id.eq(user_id)))
        .set(users::discord_id.eq(Some("550348841396994048".to_string())))
        .execute(&mut conn)
        .unwrap();

    let level_uuid = create_test_level(&mut conn).await;
    diesel::update(levels::table.filter(levels::id.eq(level_uuid)))
        .set(levels::level_id.eq(12345))
        .execute(&mut conn)
        .unwrap();

    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::post()
        .uri("/arepl/records/pemonlist/sync")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
}
