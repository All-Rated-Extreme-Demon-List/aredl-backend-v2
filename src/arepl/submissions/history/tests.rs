#[cfg(test)]
use crate::{
    arepl::{
        levels::test_utils::create_test_level,
        submissions::{test_utils::create_test_submission},
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
    let (app, mut conn, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;

    let submission: Uuid = create_test_submission(level_id, user_id, &mut conn).await;

    let under_consideration_data = json!({"notes": "No way SpaceUK is hacking right guys"});

    let req = test::TestRequest::post()
        .uri(format!("/arepl/submissions/{submission}/underconsideration").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
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
    assert_eq!(arr.len(), 1);
    let entry = &arr[0];

    assert_eq!(entry["submission_id"], submission.to_string());
    assert_eq!(entry["status"], "UnderConsideration");
    assert_eq!(entry["reviewer_notes"], under_consideration_data["notes"]);
}

#[actix_web::test]
async fn get_record_submission_history() {
    let (app, mut conn, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;

    let submission: Uuid = create_test_submission(level_id, user_id, &mut conn).await;

    let under_consideration_data = json!({"notes": "Under Consideration note"});

    let req = test::TestRequest::post()
        .uri(format!("/arepl/submissions/{submission}/underconsideration").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&under_consideration_data)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );

    let deny_data = json!({"notes": "Deny note"});

    let req = test::TestRequest::post()
        .uri(format!("/arepl/submissions/{submission}/deny").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&deny_data)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );

    let accept_data = json!({"notes": "Accept note"});

    let req = test::TestRequest::post()
        .uri(format!("/arepl/submissions/{submission}/accept").as_str())
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
    let record_id = record_data["id"].as_str().unwrap().to_string();

    let req = test::TestRequest::get()
        .uri(format!("/arepl/submissions/{record_id}/history?is_record_id=true").as_str())
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
    println!("{:?}", arr);
    assert_eq!(arr.len(), 3);
    let accept_log = &arr[0];

    assert_eq!(accept_log["submission_id"], submission.to_string());
    assert_eq!(accept_log["status"], "Accepted");
    assert_eq!(accept_log["reviewer_notes"], accept_data["notes"]);
    assert_eq!(accept_log["record_id"], record_id);
}