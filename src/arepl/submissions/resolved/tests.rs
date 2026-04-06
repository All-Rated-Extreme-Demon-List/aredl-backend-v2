#[cfg(test)]
use {
    crate::{
        arepl::{
            levels::test_utils::create_test_level,
            submissions::test_utils::{
                create_test_submission, create_two_test_submissions_with_different_timestamps,
            },
        },
        auth::{create_test_token, Permission},
        schema::arepl::submissions,
        test_utils::*,
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
};

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
        .uri(&format!("/arepl/submissions/{submission}"))
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
        .uri("/arepl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        403,
        Some("You do not have the required permission (submission_review_full) to access this endpoint"),
    )
    .await;
}

#[actix_web::test]
async fn resolved_find_all() {
    let (app, db, auth, _) = init_test_app().await;
    let (mod_user, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
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
async fn resolved_find_all_base_reviewer_forbidden() {
    let (app, db, auth, _) = init_test_app().await;
    let (base_reviewer, _) = create_test_user(&db, Some(Permission::SubmissionReviewBase)).await;
    let token = create_test_token(base_reviewer, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::get()
        .uri("/arepl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_error_response(
        resp,
        403,
        Some("You do not have the required permission (submission_review_full) to access this endpoint"),
    )
    .await;
}

#[actix_web::test]
async fn resolved_find_all_reviewer_filter_hides_base_reviewer_for_non_auditor() {
    let (app, db, auth, _) = init_test_app().await;
    let (owner, _) = create_test_user(&db, None).await;
    let (base_reviewer, _) = create_test_user(&db, Some(Permission::SubmissionReviewBase)).await;
    let (full_non_auditor, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;

    let token = create_test_token(full_non_auditor, &auth.jwt_encoding_key).unwrap();

    let level = create_test_level(&db).await;
    let submission = create_test_submission(level, owner, &db).await;

    diesel::update(submissions::table.filter(submissions::id.eq(submission)))
        .set(submissions::reviewer_id.eq::<Option<uuid::Uuid>>(Some(base_reviewer)))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = test::TestRequest::get()
        .uri(format!("/arepl/submissions?reviewer_filter={}", base_reviewer).as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[actix_web::test]
async fn resolved_find_all_redacts_base_reviewer_but_auditor_can_filter_and_see() {
    let (app, db, auth, _) = init_test_app().await;
    let (owner, _) = create_test_user(&db, None).await;
    let (base_reviewer, _) = create_test_user(&db, Some(Permission::SubmissionReviewBase)).await;
    let (full_non_auditor, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let (auditor, _) = create_test_user(&db, Some(Permission::ReviewersAudit)).await;

    let full_token = create_test_token(full_non_auditor, &auth.jwt_encoding_key).unwrap();
    let auditor_token = create_test_token(auditor, &auth.jwt_encoding_key).unwrap();

    let level = create_test_level(&db).await;
    let submission = create_test_submission(level, owner, &db).await;

    diesel::update(submissions::table.filter(submissions::id.eq(submission)))
        .set(submissions::reviewer_id.eq::<Option<uuid::Uuid>>(Some(base_reviewer)))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = test::TestRequest::get()
        .uri("/arepl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", full_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let entry = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == submission.to_string())
        .unwrap();
    assert_eq!(
        entry["reviewer"]["id"],
        "00000000-0000-0000-0000-000000000000"
    );
    assert_eq!(entry["reviewer"]["username"], "Hidden user");

    let req = test::TestRequest::get()
        .uri(format!("/arepl/submissions?reviewer_filter={}", base_reviewer).as_str())
        .insert_header(("Authorization", format!("Bearer {}", auditor_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let entry = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["id"] == submission.to_string())
        .unwrap();
    assert_eq!(entry["reviewer"]["id"], base_reviewer.to_string());
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

#[actix_web::test]
async fn resolved_find_one_hides_private_fields_for_base_reviewer() {
    let (app, db, auth, _) = init_test_app().await;
    let (owner, _) = create_test_user(&db, None).await;
    let (full_reviewer, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let (base_reviewer, _) = create_test_user(&db, Some(Permission::SubmissionReviewBase)).await;
    let full_token = create_test_token(full_reviewer, &auth.jwt_encoding_key).unwrap();
    let base_token = create_test_token(base_reviewer, &auth.jwt_encoding_key).unwrap();

    let level = create_test_level(&db).await;
    let submission = create_test_submission(level, owner, &db).await;

    let patch_req = test::TestRequest::patch()
        .uri(&format!("/arepl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", full_token)))
        .set_json(&serde_json::json!({
            "status": "UnderConsideration",
            "reviewer_notes": "public note",
            "private_reviewer_notes": "private note"
        }))
        .to_request();
    let patch_resp = test::call_service(&app, patch_req).await;
    assert!(
        patch_resp.status().is_success(),
        "status is {}",
        patch_resp.status()
    );

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", base_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    assert_eq!(body["id"], submission.to_string());
    assert_eq!(body["reviewer_notes"], "public note");
    assert!(body.get("reviewer").is_none());
    assert!(body.get("private_reviewer_notes").is_none());
}

#[actix_web::test]
async fn resolved_find_one_hides_base_reviewer_for_non_auditor_but_not_for_auditor() {
    let (app, db, auth, _) = init_test_app().await;
    let (owner, _) = create_test_user(&db, None).await;
    let (base_reviewer, _) = create_test_user(&db, Some(Permission::SubmissionReviewBase)).await;
    let (full_non_auditor, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let (auditor, _) = create_test_user(&db, Some(Permission::ReviewersAudit)).await;

    let full_token = create_test_token(full_non_auditor, &auth.jwt_encoding_key).unwrap();
    let auditor_token = create_test_token(auditor, &auth.jwt_encoding_key).unwrap();

    let level = create_test_level(&db).await;
    let submission = create_test_submission(level, owner, &db).await;

    diesel::update(submissions::table.filter(submissions::id.eq(submission)))
        .set((
            submissions::reviewer_id.eq::<Option<uuid::Uuid>>(Some(base_reviewer)),
            submissions::private_reviewer_notes.eq::<Option<String>>(Some("private".to_string())),
        ))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", full_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body.get("reviewer").is_none());
    assert_eq!(body["private_reviewer_notes"], "private");

    let req = test::TestRequest::get()
        .uri(&format!("/arepl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", auditor_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["reviewer"]["id"], base_reviewer.to_string());
}

#[actix_web::test]
async fn resolved_find_all_sort_oldest_created_at() {
    let (app, db, auth, _) = init_test_app().await;
    let (mod_user, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let token = create_test_token(mod_user, &auth.jwt_encoding_key).unwrap();

    let (older, newer) = create_two_test_submissions_with_different_timestamps(&db, mod_user).await;

    let req = test::TestRequest::get()
        .uri("/arepl/submissions?per_page=10&sort=OldestCreatedAt")
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
    let (mod_user, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let token = create_test_token(mod_user, &auth.jwt_encoding_key).unwrap();

    let (older, newer) = create_two_test_submissions_with_different_timestamps(&db, mod_user).await;

    let req = test::TestRequest::get()
        .uri("/arepl/submissions?per_page=10&sort=NewestCreatedAt")
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
    let (mod_user, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let token = create_test_token(mod_user, &auth.jwt_encoding_key).unwrap();

    let (older, newer) = create_two_test_submissions_with_different_timestamps(&db, mod_user).await;

    let req = test::TestRequest::get()
        .uri("/arepl/submissions?per_page=10&sort=OldestUpdatedAt")
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
    let (mod_user, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let token = create_test_token(mod_user, &auth.jwt_encoding_key).unwrap();

    let (older, newer) = create_two_test_submissions_with_different_timestamps(&db, mod_user).await;

    let req = test::TestRequest::get()
        .uri("/arepl/submissions?per_page=10&sort=NewestUpdatedAt")
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
async fn resolved_find_all_sort_shortest_completion_time() {
    let (app, db, auth, _) = init_test_app().await;
    let (mod_user, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let token = create_test_token(mod_user, &auth.jwt_encoding_key).unwrap();

    let (slower, faster) =
        create_two_test_submissions_with_different_timestamps(&db, mod_user).await;

    let req = test::TestRequest::get()
        .uri("/arepl/submissions?per_page=10&sort=ShortestCompletionTime")
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
    assert_eq!(got[0], faster.to_string());
    assert_eq!(got[1], slower.to_string());
}

#[actix_web::test]
async fn resolved_find_all_sort_longest_completion_time() {
    let (app, db, auth, _) = init_test_app().await;
    let (mod_user, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let token = create_test_token(mod_user, &auth.jwt_encoding_key).unwrap();

    let (slower, faster) =
        create_two_test_submissions_with_different_timestamps(&db, mod_user).await;

    let req = test::TestRequest::get()
        .uri("/arepl/submissions?per_page=10&sort=LongestCompletionTime")
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
    assert_eq!(got[0], slower.to_string());
    assert_eq!(got[1], faster.to_string());
}
