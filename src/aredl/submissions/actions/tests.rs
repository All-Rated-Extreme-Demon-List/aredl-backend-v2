#[cfg(test)]
use crate::{
    aredl::{
        levels::test_utils::create_test_level, submissions::test_utils::create_test_submission,
    },
    auth::{create_test_token, Permission},
    schema::{aredl::records, shifts},
    shifts::{test_utils::create_test_shift, ShiftStatus},
    test_utils::*,
    users::test_utils::create_test_user,
};
#[cfg(test)]
use actix_web::test::{self, read_body_json};
#[cfg(test)]
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
#[cfg(test)]
use serde_json::json;
#[cfg(test)]
use uuid::Uuid;

#[actix_web::test]
async fn accept_submission() {
    let (app, mut conn, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;

    let submission: Uuid = create_test_submission(level_id, user_id, &mut conn).await;

    let accept_data = json!({"notes": "GG!"});

    let req = test::TestRequest::post()
        .uri(format!("/aredl/submissions/{submission}/accept").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&accept_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "status of req is {}",
        resp.status()
    );

    let record_string = records::table
        .filter(records::level_id.eq(level_id))
        .filter(records::submitted_by.eq(user_id))
        .select(records::reviewer_notes)
        .first::<Option<String>>(&mut conn)
        .expect("Failed to get new record!")
        .unwrap();
    let new_record = record_string.as_str();

    assert_eq!(
        new_record,
        accept_data["notes"].as_str().unwrap(),
        "Reviewer notes do not match!"
    )
}

#[actix_web::test]
async fn deny_submission() {
    let (app, mut conn, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;

    let submission: Uuid = create_test_submission(level_id, user_id, &mut conn).await;

    let deny_data = json!({"notes": "No Cheat Indicator:tm:"});

    let req = test::TestRequest::post()
        .uri(format!("/aredl/submissions/{submission}/deny").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&deny_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "status of req is {}",
        resp.status()
    );

    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(
        body["reviewer_notes"], deny_data["notes"],
        "Reviewer notes do not match!"
    );
    assert_eq!(
        body["status"].as_str().unwrap(),
        "Denied",
        "Submission is not denied!"
    );
}

#[actix_web::test]
async fn submission_under_consideration() {
    let (app, mut conn, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;

    let submission: Uuid = create_test_submission(level_id, user_id, &mut conn).await;

    let under_consideration_data = json!({"notes": "No way SpaceUK is hacking right guys"});

    let req = test::TestRequest::post()
        .uri(format!("/aredl/submissions/{submission}/underconsideration").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&under_consideration_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "status of req is {}",
        resp.status()
    );

    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(
        body["reviewer_notes"], under_consideration_data["notes"],
        "Reviewer notes do not match!"
    );
    assert_eq!(
        body["status"].as_str().unwrap(),
        "UnderConsideration",
        "Submission is not denied!"
    );
}

#[actix_web::test]
async fn actions_increment_shift() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (mod_id, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token_mod = create_test_token(mod_id, &auth.jwt_encoding_key).unwrap();
    let shift_id = create_test_shift(&mut conn, mod_id, true).await;
    let level = create_test_level(&mut conn).await;
    let _submission = create_test_submission(level, mod_id, &mut conn).await;

    let req = test::TestRequest::get()
        .uri("/aredl/submissions/claim")
        .insert_header(("Authorization", format!("Bearer {}", token_mod)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    let sub_id = body["id"].as_str().unwrap().to_string();

    let notes = json!({"notes":"ok"});
    let req = test::TestRequest::post()
        .uri(&format!("/aredl/submissions/{sub_id}/accept"))
        .insert_header(("Authorization", format!("Bearer {}", token_mod)))
        .set_json(&notes)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let count: i32 = shifts::table
        .find(shift_id)
        .select(shifts::completed_count)
        .first(&mut conn)
        .unwrap();
    assert_eq!(count, 1);
}

#[actix_web::test]
async fn unclaim_submission() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (mod_id, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token_mod = create_test_token(mod_id, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&mut conn).await;
    let submission_id = create_test_submission(level, mod_id, &mut conn).await;

    let req = test::TestRequest::get()
        .uri("/aredl/submissions/claim")
        .insert_header(("Authorization", format!("Bearer {}", token_mod)))
        .to_request();
    let _ = test::call_service(&app, req).await;

    let req = test::TestRequest::post()
        .uri(&format!("/aredl/submissions/{submission_id}/unclaim"))
        .insert_header(("Authorization", format!("Bearer {}", token_mod)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["status"], "Pending");
}

#[actix_web::test]
async fn accept_existing_record() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (mod_id, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token_mod = create_test_token(mod_id, &auth.jwt_encoding_key).unwrap();
    let (user_id, _) = create_test_user(&mut conn, None).await;
    let level = create_test_level(&mut conn).await;
    let existing_record =
        crate::aredl::records::test_utils::create_test_record(&mut conn, user_id, level).await;
    let submission = create_test_submission(level, user_id, &mut conn).await;
    let req = test::TestRequest::get()
        .uri("/aredl/submissions/claim")
        .insert_header(("Authorization", format!("Bearer {}", token_mod)))
        .to_request();
    let _ = test::call_service(&app, req).await;
    let notes = json!({"notes":"hi"});
    let req = test::TestRequest::post()
        .uri(&format!("/aredl/submissions/{submission}/accept"))
        .insert_header(("Authorization", format!("Bearer {}", token_mod)))
        .set_json(&notes)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["id"].as_str().unwrap(), existing_record.to_string());
}

#[actix_web::test]
async fn shift_completes_after_accept() {
    use diesel::{ExpressionMethods, RunQueryDsl};

    let (app, mut conn, auth, _) = init_test_app().await;
    let (mod_id, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod_id, &auth.jwt_encoding_key).unwrap();
    let shift_id = create_test_shift(&mut conn, mod_id, true).await;
    diesel::update(shifts::table.filter(shifts::id.eq(shift_id)))
        .set(shifts::target_count.eq(1))
        .execute(&mut conn)
        .unwrap();
    let level = create_test_level(&mut conn).await;
    let _sub = create_test_submission(level, mod_id, &mut conn).await;

    let req = test::TestRequest::get()
        .uri("/aredl/submissions/claim")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    let sub_id = body["id"].as_str().unwrap();

    let req = test::TestRequest::post()
        .uri(&format!("/aredl/submissions/{sub_id}/accept"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let status: ShiftStatus = shifts::table
        .find(shift_id)
        .select(shifts::status)
        .first(&mut conn)
        .unwrap();
    assert_eq!(status, ShiftStatus::Completed);
}

#[actix_web::test]
async fn deny_already_denied() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (mod_id, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod_id, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&mut conn).await;
    let _submission = create_test_submission(level, mod_id, &mut conn).await;

    let req = test::TestRequest::get()
        .uri("/aredl/submissions/claim")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    let sub_id = body["id"].as_str().unwrap().to_string();
    let notes = json!({"notes":"hi"});
    let req = test::TestRequest::post()
        .uri(&format!("/aredl/submissions/{sub_id}/deny"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&notes)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::post()
        .uri(&format!("/aredl/submissions/{sub_id}/deny"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&notes)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn under_consideration_already_uc() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (mod_id, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod_id, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&mut conn).await;
    let _submission = create_test_submission(level, mod_id, &mut conn).await;

    let req = test::TestRequest::get()
        .uri("/aredl/submissions/claim")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    let sub_id = body["id"].as_str().unwrap().to_string();
    let notes = json!({"notes":"ok"});
    let req = test::TestRequest::post()
        .uri(&format!("/aredl/submissions/{sub_id}/underconsideration"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&notes)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::post()
        .uri(&format!("/aredl/submissions/{sub_id}/underconsideration"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&notes)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn unclaim_not_claimed() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (mod_id, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod_id, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&mut conn).await;
    let submission_id = create_test_submission(level, mod_id, &mut conn).await;

    let req = test::TestRequest::post()
        .uri(&format!("/aredl/submissions/{submission_id}/unclaim"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}
