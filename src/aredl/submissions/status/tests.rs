#[cfg(test)]
use crate::{
    aredl::{levels::test_utils::create_test_level, submissions::status::SubmissionsEnabled},
    auth::{create_test_token, Permission},
};
#[cfg(test)]
use crate::{test_utils::*, users::test_utils::create_test_user};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn enable_submissions() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    SubmissionsEnabled::disable(&mut db.connection().unwrap(), user_id)
        .expect("Failed to temporarily disable submissions");

    let req = test::TestRequest::post()
        .uri("/aredl/submissions/status/enable")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let status = SubmissionsEnabled::is_enabled(&mut db.connection().unwrap())
        .expect("Failed to get submission status");

    assert_eq!(status, true)
}

#[actix_web::test]
async fn disable_submissions() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let req = test::TestRequest::post()
        .uri("/aredl/submissions/status/disable")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let status = SubmissionsEnabled::is_enabled(&mut db.connection().unwrap())
        .expect("Failed to get submission status");

    assert_eq!(status, false);

    let level_id = create_test_level(&db).await;

    let data = json!({
        "level_id": level_id,
        "video_url": "https://video.com",
        "raw_url": "https://raw.com"
    });

    let req = test::TestRequest::post()
        .uri("/aredl/submissions/")
        .set_json(data)
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_client_error(),
        "status is {}",
        resp.status()
    );
}

#[actix_web::test]
async fn get_submission_status() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    SubmissionsEnabled::disable(&mut db.connection().unwrap(), user_id)
        .expect("Failed to temporarily disable submissions");

    let req = test::TestRequest::get()
        .uri("/aredl/submissions/status/")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let status: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(status.as_bool().unwrap(), false);

    SubmissionsEnabled::enable(&mut db.connection().unwrap(), user_id)
        .expect("Failed to temporarily disable submissions");

    let req = test::TestRequest::get()
        .uri("/aredl/submissions/status/")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let status: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(status.as_bool().unwrap(), true);
}

#[actix_web::test]
async fn get_submission_status_full() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    SubmissionsEnabled::disable(&mut db.connection().unwrap(), user_id)
        .expect("Failed to temporarily disable submissions");

    let req = test::TestRequest::get()
        .uri("/aredl/submissions/status/full")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(body["moderator"]["id"], user_id.to_string());
    assert_eq!(body["enabled"], false);
}

#[actix_web::test]
async fn get_submission_status_history() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::ShiftManage)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    SubmissionsEnabled::disable(&mut db.connection().unwrap(), user_id)
        .expect("Failed to temporarily disable submissions");
    SubmissionsEnabled::enable(&mut db.connection().unwrap(), user_id)
        .expect("Failed to temporarily enable submissions");

    let req = test::TestRequest::get()
        .uri("/aredl/submissions/status/history")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(body[0]["enabled"], true);
    assert_eq!(body[1]["enabled"], false);
    assert!(body
        .as_array()
        .unwrap()
        .iter()
        .all(|s| s["moderator"]["id"] == user_id.to_string()))
}
