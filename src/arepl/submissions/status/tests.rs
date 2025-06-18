#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission},
    arepl::{levels::test_utils::create_test_level, submissions::status::SubmissionsEnabled}
};
#[cfg(test)]
use crate::{test_utils::*, users::test_utils::create_test_user};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn enable_submissions() {
    let (app, mut conn, auth) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    SubmissionsEnabled::disable(&mut conn, user_id).expect("Failed to temporarily disable submissions");

    let req = test::TestRequest::post()
        .uri("/arepl/submissions/status/enable")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let status = SubmissionsEnabled::is_enabled(&mut conn).expect("Failed to get submission status");

    assert_eq!(status, true)
}

#[actix_web::test]
async fn disable_submissions() {
    let (app, mut conn, auth) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let req = test::TestRequest::post()
        .uri("/arepl/submissions/status/disable")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let status = SubmissionsEnabled::is_enabled(&mut conn).expect("Failed to get submission status");

    assert_eq!(status, false);

    let level_id = create_test_level(&mut conn).await;

    let data = json!({
        "level_id": level_id,
        "video_url": "https://video.com",
        "raw_url": "https://raw.com"
    });

    let req = test::TestRequest::post()
        .uri("/arepl/submissions/")
        .set_json(data)
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error(), "status is {}", resp.status());
}

#[actix_web::test]
async fn get_submission_status() {
    let (app, mut conn, auth) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    SubmissionsEnabled::disable(&mut conn, user_id).expect("Failed to temporarily disable submissions");

    let req = test::TestRequest::get()
        .uri("/arepl/submissions/status/")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let status: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(status.as_bool().unwrap(), false);


    SubmissionsEnabled::enable(&mut conn, user_id).expect("Failed to temporarily disable submissions");

    let req = test::TestRequest::get()
        .uri("/arepl/submissions/status/")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let status: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(status.as_bool().unwrap(), true);
}

#[actix_web::test]
async fn get_submission_status_full() {
    let (app, mut conn, auth) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    SubmissionsEnabled::disable(&mut conn, user_id).expect("Failed to temporarily disable submissions");

    let req = test::TestRequest::get()
        .uri("/arepl/submissions/status/full")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(body["moderator"], user_id.to_string());
    assert_eq!(body["enabled"], false);
}

#[actix_web::test]
async fn get_submission_status_history() {
    let (app, mut conn, auth) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    SubmissionsEnabled::disable(&mut conn, user_id).expect("Failed to temporarily disable submissions");
    SubmissionsEnabled::enable(&mut conn, user_id).expect("Failed to temporarily enable submissions");

    let req = test::TestRequest::get()
        .uri("/arepl/submissions/status/history")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(body[0]["enabled"], true);
    assert_eq!(body[1]["enabled"], false);
    assert!(
        body.as_array().unwrap()
            .iter().all(|s| s["moderator"] == user_id.to_string())
    )
}