#[cfg(test)]
use {
    crate::arepl::levels::test_utils::create_test_level,
    crate::arepl::records::test_utils::create_test_record,
    crate::auth::create_test_token,
    crate::schema::{arepl::levels, arepl::records, users},
    crate::{test_utils::*, users::test_utils::create_test_user},
    actix_web::test::{self, read_body_json},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    httpmock::prelude::*,
    serde_json::json,
    serial_test::serial,
};

#[actix_web::test]
#[serial]
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

    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;

    diesel::update(users::table.filter(users::id.eq(user_id)))
        .set(users::discord_id.eq(Some("550348841396994048".to_string())))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let level_uuid = create_test_level(&db).await;
    diesel::update(levels::table.filter(levels::id.eq(level_uuid)))
        .set(levels::level_id.eq(12345))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::post()
        .uri("/arepl/submissions/pemonlist/sync")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
}

#[actix_web::test]
#[serial]
async fn sync_pemonlist_preserves_verification_flag() {
    // Mock pemonlist response
    let server = MockServer::start_async().await;
    let response_body = json!({
        "records": [
            {
                "formatted_time": "00:00:05.100",
                "level": {"level_id": 54321},
                "mobile": true,
                "video_id": "wxyz"
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

    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;

    diesel::update(users::table.filter(users::id.eq(user_id)))
        .set(users::discord_id.eq(Some("550348841396994048".to_string())))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let level_uuid = create_test_level(&db).await;
    diesel::update(levels::table.filter(levels::id.eq(level_uuid)))
        .set(levels::level_id.eq(54321))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let record_id = create_test_record(&db, user_id, level_uuid).await;
    diesel::update(records::table.filter(records::id.eq(record_id)))
        .set(records::is_verification.eq(true))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let req = test::TestRequest::post()
        .uri("/arepl/submissions/pemonlist/sync")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let is_verification_after: bool = records::table
        .filter(records::id.eq(record_id))
        .select(records::is_verification)
        .first(&mut db.connection().unwrap())
        .unwrap();
    assert!(
        is_verification_after,
        "verification flag should be preserved"
    );
}
