#[cfg(test)]
use {
    crate::arepl::levels::test_utils::{create_test_level, set_test_level_gd_id},
    crate::arepl::records::test_utils::{
        create_test_record, get_test_record, set_test_record_verification,
    },
    crate::auth::create_test_token,
    crate::{
        test_utils::*,
        users::test_utils::{create_test_user, set_test_user_discord_id},
    },
    actix_web::test::{self, read_body_json},
    httpmock::prelude::*,
    serde_json::{json, Value},
    serial_test::serial,
};

#[actix_web::test]
#[serial]
async fn sync_pemonlist() {
    let server = MockServer::start_async().await;
    let response_body = json!({
        "records": [
            {
                "formatted_time": "29:00:05.100",
                "level": {"level_id": 12345},
                "mobile": false,
                "video_id": "abcdefghijk"
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

    set_test_user_discord_id(&db, user_id, "550348841396994048").await;

    let level_uuid = create_test_level(&db).await;
    set_test_level_gd_id(&db, level_uuid, 12345).await;

    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::post()
        .uri("/arepl/submissions/pemonlist/sync")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    let arr = body.as_array().expect("expected JSON array");
    assert_eq!(arr.len(), 1);

    let submission = &arr[0];

    assert_eq!(
        submission.get("completion_time").and_then(Value::as_i64),
        Some(104_405_100),
        "unexpected completion_time: {}",
        submission
            .get("completion_time")
            .unwrap_or(&serde_json::Value::Null)
    );

    assert_eq!(
        submission.get("mobile").and_then(Value::as_bool),
        Some(false)
    );
    assert_eq!(
        submission.get("video_url").and_then(Value::as_str),
        Some("https://www.youtube.com/watch?v=abcdefghijk")
    );
    assert_eq!(
        submission.get("status").and_then(Value::as_str),
        Some("Accepted")
    );
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

    set_test_user_discord_id(&db, user_id, "550348841396994048").await;

    let level_uuid = create_test_level(&db).await;
    set_test_level_gd_id(&db, level_uuid, 54321).await;

    let record_id = create_test_record(&db, user_id, level_uuid).await;
    set_test_record_verification(&db, record_id, true).await;

    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let req = test::TestRequest::post()
        .uri("/arepl/submissions/pemonlist/sync")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let is_verification_after = get_test_record(&db, record_id).is_verification;
    assert!(
        is_verification_after,
        "verification flag should be preserved"
    );
}
