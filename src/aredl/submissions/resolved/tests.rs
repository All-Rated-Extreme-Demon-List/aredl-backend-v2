#[cfg(test)]
use {
    crate::{
        aredl::{
            levels::test_utils::create_test_level,
            submissions::test_utils::{
                create_test_submission, create_two_test_submissions_with_different_timestamps,
            },
        },
        auth::{create_test_token, Permission},
        test_utils::*,
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
};
#[actix_web::test]
async fn resolved_find_me_and_filters() {
    let (app, db, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&db).await;
    let submission = create_test_submission(level, user, &db).await;
    let req = test::TestRequest::get()
        .uri("/aredl/submissions/@me?status_filter=Pending")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["id"], submission.to_string());
}

#[actix_web::test]
async fn resolved_find_one_unauthorized() {
    let (app, db, auth, _) = init_test_app().await;
    let (user1, _) = create_test_user(&db, None).await;
    let (user2, _) = create_test_user(&db, None).await;
    let token2 = create_test_token(user2, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&db).await;
    let submission = create_test_submission(level, user1, &db).await;

    let req = test::TestRequest::get()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token2)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(resp, 404, Some("Record not found")).await;
}

#[actix_web::test]
async fn resolved_find_all_requires_auth() {
    let (app, db, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let req = test::TestRequest::get()
        .uri("/aredl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        403,
        Some("You do not have the required permission (submission_review) to access this endpoint"),
    )
    .await;
}

#[actix_web::test]
async fn resolved_find_all() {
    let (app, db, auth, _) = init_test_app().await;
    let (mod_user, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod_user, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&db).await;
    create_test_submission(level, mod_user, &db).await;

    let req = test::TestRequest::get()
        .uri("/aredl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn resolved_find_own() {
    let (app, db, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&db).await;
    let submission = create_test_submission(level, user, &db).await;

    let req = test::TestRequest::get()
        .uri("/aredl/submissions/@me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|s| s["id"] == submission.to_string()));
}

#[actix_web::test]
async fn resolved_find_all_sort_oldest_created_at() {
    let (app, db, auth, _) = init_test_app().await;
    let (mod_user, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod_user, &auth.jwt_encoding_key).unwrap();
    let (older, newer) = create_two_test_submissions_with_different_timestamps(&db, mod_user).await;

    let req = test::TestRequest::get()
        .uri("/aredl/submissions?per_page=10&sort=OldestCreatedAt")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let got: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_string())
        .collect();

    assert!(got.len() >= 2);
    assert_eq!(got[0], older.to_string());
    assert_eq!(got[1], newer.to_string());
}

#[actix_web::test]
async fn resolved_find_all_sort_newest_created_at() {
    let (app, db, auth, _) = init_test_app().await;
    let (mod_user, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod_user, &auth.jwt_encoding_key).unwrap();

    let (older, newer) = create_two_test_submissions_with_different_timestamps(&db, mod_user).await;

    let req = test::TestRequest::get()
        .uri("/aredl/submissions?per_page=10&sort=NewestCreatedAt")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let got: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_string())
        .collect();

    assert!(got.len() >= 2);
    assert_eq!(got[0], newer.to_string());
    assert_eq!(got[1], older.to_string());
}

#[actix_web::test]
async fn resolved_find_all_sort_oldest_updated_at() {
    let (app, db, auth, _) = init_test_app().await;
    let (mod_user, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod_user, &auth.jwt_encoding_key).unwrap();

    let (older, newer) = create_two_test_submissions_with_different_timestamps(&db, mod_user).await;

    let req = test::TestRequest::get()
        .uri("/aredl/submissions?per_page=10&sort=OldestUpdatedAt")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let got: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_string())
        .collect();

    assert!(got.len() >= 2);
    assert_eq!(got[0], older.to_string());
    assert_eq!(got[1], newer.to_string());
}

#[actix_web::test]
async fn resolved_find_all_sort_newest_updated_at() {
    let (app, db, auth, _) = init_test_app().await;
    let (mod_user, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod_user, &auth.jwt_encoding_key).unwrap();

    let (older, newer) = create_two_test_submissions_with_different_timestamps(&db, mod_user).await;

    let req = test::TestRequest::get()
        .uri("/aredl/submissions?per_page=10&sort=NewestUpdatedAt")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let got: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v["id"].as_str().unwrap().to_string())
        .collect();

    assert!(got.len() >= 2);
    assert_eq!(got[0], newer.to_string());
    assert_eq!(got[1], older.to_string());
}
