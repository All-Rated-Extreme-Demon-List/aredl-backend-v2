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
use actix_web::test;
#[cfg(test)]
use serde_json::json;
#[cfg(test)]
use uuid::Uuid;

#[actix_web::test]
async fn get_submission_history() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let (moderator_id, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let user_token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let moderator_token =
        create_test_token(moderator_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;

    let submission: Uuid = create_test_submission(level_id, user_id, &db).await;

    let under_consideration_data = json!({"status": "UnderConsideration", "reviewer_notes": "No way SpaceUK is hacking right guys"});

    let req = test::TestRequest::patch()
        .uri(format!("/arepl/submissions/{submission}").as_str())
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
        .uri(format!("/arepl/submissions/{submission}/history").as_str())
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );

    let body: serde_json::Value = test::read_body_json(res).await;

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
    let (moderator_id, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token =
        create_test_token(moderator_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;

    let submission: Uuid = create_test_submission(level_id, user_id, &db).await;

    let under_consideration_data =
        json!({"status": "UnderConsideration", "reviewer_notes": "Under Consideration note"});

    let req = test::TestRequest::patch()
        .uri(format!("/arepl/submissions/{submission}").as_str())
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
        .uri(format!("/arepl/submissions/{submission}").as_str())
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
        .uri(format!("/arepl/submissions/{submission}").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&accept_data)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );

    let record_data: serde_json::Value = test::read_body_json(res).await;
    let submission_id = record_data["id"].as_str().unwrap().to_string();

    let req = test::TestRequest::get()
        .uri(format!("/arepl/submissions/{submission_id}/history").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );

    let body: serde_json::Value = test::read_body_json(res).await;

    let arr = body.as_array().unwrap();

    assert_eq!(arr.len(), 4);
    let accept_log = &arr[0];

    assert_eq!(accept_log["submission_id"], submission.to_string());
    assert_eq!(accept_log["status"], "Accepted");
    assert_eq!(accept_log["reviewer_notes"], accept_data["reviewer_notes"]);
    assert_eq!(accept_log["submission_id"], submission_id);
}
