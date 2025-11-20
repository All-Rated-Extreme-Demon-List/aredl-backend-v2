#[cfg(test)]
use crate::{
    arepl::{
        levels::test_utils::create_test_level, submissions::test_utils::create_test_submission,
    },
    auth::{create_test_token, Permission},
    test_utils::*,
    users::test_utils::create_test_user,
};
#[cfg(test)]
use actix_web::test::{self, read_body_json};

#[actix_web::test]
async fn resolved_find_me_and_filters() {
    let (app, db, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&db).await;
    let submission = create_test_submission(level, user, &db).await;
    let req = test::TestRequest::get()
        .uri("/arepl/submissions/@me?status_filter=Pending")
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
        .uri(&format!("/arepl/submissions/resolved/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token2)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn resolved_find_all_requires_auth() {
    let (app, _, _, _) = init_test_app().await;
    let req = test::TestRequest::get()
        .uri("/arepl/submissions/resolved")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn resolved_find_all() {
    let (app, db, auth, _) = init_test_app().await;
    let (mod_user, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod_user, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&db).await;
    create_test_submission(level, mod_user, &db).await;

    let req = test::TestRequest::get()
        .uri("/arepl/submissions")
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
        .uri("/arepl/submissions/@me")
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
