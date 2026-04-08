#[cfg(test)]
use {
    crate::{
        aredl::{
            levels::test_utils::create_test_level, submissions::test_utils::create_test_submission,
        },
        auth::{create_test_token, Permission},
        schema::aredl::submission_history,
        test_utils::*,
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
    serde_json::json,
    uuid::Uuid,
};

#[actix_web::test]
async fn get_submission_history() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let (moderator_id, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let user_token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let moderator_token =
        create_test_token(moderator_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;

    let submission: Uuid = create_test_submission(level_id, user_id, &db).await;

    let under_consideration_data = json!({"status": "UnderConsideration", "reviewer_notes": "No way SpaceUK is hacking right guys"});

    let req = test::TestRequest::patch()
        .uri(format!("/aredl/submissions/{submission}").as_str())
        .insert_header(("Authorization", format!("Bearer {}", moderator_token)))
        .set_json(&under_consideration_data)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );

    let req = test::TestRequest::get()
        .uri(format!("/aredl/submissions/{submission}/history").as_str())
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );

    let body: serde_json::Value = read_body_json(res).await;

    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    let last_entry = &arr[0];

    assert_eq!(last_entry["submission_id"], submission.to_string());
    assert_eq!(last_entry["status"], "UnderConsideration");
    assert_eq!(
        last_entry["reviewer_notes"],
        under_consideration_data["reviewer_notes"]
    );

    let first_entry = &arr[1];

    assert_eq!(first_entry["submission_id"], submission.to_string());
    assert_eq!(first_entry["status"], "Pending");
}

#[actix_web::test]
async fn get_full_submission_history() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let (moderator_id, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let token =
        create_test_token(moderator_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;

    let submission: Uuid = create_test_submission(level_id, user_id, &db).await;

    let under_consideration_data =
        json!({"status": "UnderConsideration", "reviewer_notes": "Under Consideration note"});

    let req = test::TestRequest::patch()
        .uri(format!("/aredl/submissions/{submission}").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&under_consideration_data)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );

    let deny_data = json!({"status": "Denied", "reviewer_notes": "Deny note"});

    let req = test::TestRequest::patch()
        .uri(format!("/aredl/submissions/{submission}").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&deny_data)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );

    let accept_data = json!({"status": "Accepted", "reviewer_notes": "Accept note"});

    let req = test::TestRequest::patch()
        .uri(format!("/aredl/submissions/{submission}").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&accept_data)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );

    let record_data: serde_json::Value = read_body_json(res).await;
    let submission_id = record_data["id"].as_str().unwrap().to_string();

    let req = test::TestRequest::get()
        .uri(format!("/aredl/submissions/{submission_id}/history").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );

    let body: serde_json::Value = read_body_json(res).await;

    let arr = body.as_array().unwrap();

    assert_eq!(arr.len(), 4);
    let accept_log = &arr[0];

    assert_eq!(accept_log["submission_id"], submission.to_string());
    assert_eq!(accept_log["status"], "Accepted");
    assert_eq!(accept_log["reviewer_notes"], accept_data["reviewer_notes"]);
    assert_eq!(accept_log["submission_id"], submission_id);
}

#[actix_web::test]
async fn get_submission_history_hides_private_fields_for_base_reviewer() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let (full_reviewer_id, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let (base_reviewer_id, _) = create_test_user(&db, Some(Permission::SubmissionReviewBase)).await;

    let full_token = create_test_token(full_reviewer_id, &auth.jwt_encoding_key)
        .expect("Failed to generate token");
    let base_token = create_test_token(base_reviewer_id, &auth.jwt_encoding_key)
        .expect("Failed to generate token");

    let level_id = create_test_level(&db).await;
    let submission: Uuid = create_test_submission(level_id, user_id, &db).await;

    let patch_data = json!({
        "status": "UnderConsideration",
        "reviewer_notes": "Visible reviewer note",
        "private_reviewer_notes": "Hidden private note"
    });

    let patch_req = test::TestRequest::patch()
        .uri(format!("/aredl/submissions/{submission}").as_str())
        .insert_header(("Authorization", format!("Bearer {}", full_token)))
        .set_json(&patch_data)
        .to_request();
    let patch_resp = test::call_service(&app, patch_req).await;
    assert!(
        patch_resp.status().is_success(),
        "status of req is {}",
        patch_resp.status()
    );

    let req = test::TestRequest::get()
        .uri(format!("/aredl/submissions/{submission}/history").as_str())
        .insert_header(("Authorization", format!("Bearer {}", base_token)))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );

    let body: serde_json::Value = read_body_json(res).await;
    let arr = body.as_array().unwrap();
    let latest = &arr[0];

    assert_eq!(latest["submission_id"], submission.to_string());
    assert_eq!(latest["status"], "UnderConsideration");
    assert_eq!(latest["reviewer_notes"], "Visible reviewer note");
    assert!(latest.get("reviewer").is_none());
    assert!(latest.get("private_reviewer_notes").is_none());
}

#[actix_web::test]
async fn get_submission_history_redacts_base_reviewer_for_non_auditor_but_not_for_auditor() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let (base_reviewer_id, _) = create_test_user(&db, Some(Permission::SubmissionReviewBase)).await;
    let (full_non_auditor_id, _) =
        create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let (auditor_id, _) = create_test_user(&db, Some(Permission::ReviewersAudit)).await;

    let full_token = create_test_token(full_non_auditor_id, &auth.jwt_encoding_key)
        .expect("Failed to generate token");
    let auditor_token =
        create_test_token(auditor_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;
    let submission: Uuid = create_test_submission(level_id, user_id, &db).await;

    let patch_data = json!({
        "status": "UnderConsideration",
        "reviewer_notes": "Visible reviewer note",
        "private_reviewer_notes": "Visible to full reviewers"
    });

    let patch_req = test::TestRequest::patch()
        .uri(format!("/aredl/submissions/{submission}").as_str())
        .insert_header(("Authorization", format!("Bearer {}", full_token)))
        .set_json(&patch_data)
        .to_request();
    let patch_resp = test::call_service(&app, patch_req).await;
    assert!(
        patch_resp.status().is_success(),
        "status of req is {}",
        patch_resp.status()
    );

    // Make the reviewer in history a base reviewer to validate redaction behavior.
    diesel::update(
        submission_history::table.filter(submission_history::submission_id.eq(submission)),
    )
    .set(submission_history::reviewer_id.eq::<Option<Uuid>>(Some(base_reviewer_id)))
    .execute(&mut db.connection().unwrap())
    .unwrap();

    let req = test::TestRequest::get()
        .uri(format!("/aredl/submissions/{submission}/history").as_str())
        .insert_header(("Authorization", format!("Bearer {}", full_token)))
        .to_request();
    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );
    let body: serde_json::Value = read_body_json(res).await;
    let latest = &body.as_array().unwrap()[0];
    assert_eq!(
        latest["reviewer"]["id"],
        "00000000-0000-0000-0000-000000000000"
    );

    let req = test::TestRequest::get()
        .uri(format!("/aredl/submissions/{submission}/history").as_str())
        .insert_header(("Authorization", format!("Bearer {}", auditor_token)))
        .to_request();
    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );
    let body: serde_json::Value = read_body_json(res).await;
    let latest = &body.as_array().unwrap()[0];
    assert_eq!(latest["reviewer"]["id"], base_reviewer_id.to_string());
}
