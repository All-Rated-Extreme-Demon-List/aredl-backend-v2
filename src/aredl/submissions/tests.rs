use crate::{
    aredl::submissions::{history::SubmissionHistory, Submission},
    schema::{
        aredl::{records, submission_history},
        shifts,
    },
    shifts::{test_utils::create_test_shift, ShiftStatus},
};
#[cfg(test)]
use crate::{
    aredl::{
        levels::test_utils::create_test_level,
        submissions::{test_utils::create_test_submission, SubmissionStatus},
    },
    auth::{create_test_token, Permission},
    schema::{aredl::submissions, roles, user_roles, users},
    test_utils::*,
    users::test_utils::create_test_user,
};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use actix_web::test::read_body_json;
#[cfg(test)]
use diesel::{ExpressionMethods, RunQueryDsl};
use diesel::{QueryDsl, SelectableHelper};
#[cfg(test)]
use serde_json::json;
#[cfg(test)]
use uuid::Uuid;

#[actix_web::test]
async fn create_submission() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;

    let submission_data = json!({
        "level_id": level_id,
        "video_url": "https://youtube.com/watch?v=xvFZjo5PgG0",
        "raw_url": "https://www.youtube.com/watch?v=xvFZjo5PgG0",
        "mobile": false
    });

    let req = test::TestRequest::post()
        .uri("/aredl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&submission_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["submitted_by"].as_str().unwrap().to_string(),
        user_id.to_string(),
        "Submitters do not match!"
    )
}

#[actix_web::test]
async fn submission_without_raw() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;

    let submission_data = json!({
        "level_id": level_id,
        "video_url": "https://youtube.com/watch?v=xvFZjo5PgG0",
        "mobile": false,
    });

    let req = test::TestRequest::post()
        .uri("/aredl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&submission_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_client_error(),
        "status is {}",
        resp.status()
    );
}

#[actix_web::test]
async fn submission_malformed_url() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;

    // video_url
    let submission_data = json!({
        "level_id": level_id,
        "video_url": "slkdfjskdlf",
        "raw_url": "https://youtube.com/watch?v=xvFZjo5PgG0",
        "mobile": false,
    });

    let req = test::TestRequest::post()
        .uri("/aredl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&submission_data)
        .to_request();

    let resp = test::call_service(&app, req).await;

    // raw_url
    let submission_data = json!({
        "level_id": level_id,
        "video_url": "https://youtube.com/watch?v=xvFZjo5PgG0",
        "raw_url": "isldjfsdkf",
        "mobile": false,
    });

    let req = test::TestRequest::post()
        .uri("/aredl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&submission_data)
        .to_request();

    let resp2 = test::call_service(&app, req).await;

    assert!(
        resp.status().is_client_error(),
        "response 1 status is {}",
        resp.status()
    );
    assert!(
        resp2.status().is_client_error(),
        "response 2 status is {}",
        resp2.status()
    );
}

#[actix_web::test]
async fn submission_edit_no_perms() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id_1, _) = create_test_user(&db, None).await;
    let token_1 =
        create_test_token(user_id_1, &auth.jwt_encoding_key).expect("Failed to generate token");

    let (user_id_2, _) = create_test_user(&db, None).await;
    let token_2 =
        create_test_token(user_id_2, &auth.jwt_encoding_key).expect("Failed to generate token");

    let (user_id_mod, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token_mod =
        create_test_token(user_id_mod, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;

    let submission_id = create_test_submission(level_id, user_id_1, &db).await;

    let submission_edit_json = json!({
        "video_url": "https://www.youtube.com/watch?v=othervideo1"
    });

    // edit own submission
    let edit_req_own = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{}", submission_id).to_string())
        .insert_header(("Authorization", format!("Bearer {}", token_1)))
        .set_json(&submission_edit_json)
        .to_request();

    let resp_edit_own = test::call_service(&app, edit_req_own).await;
    assert!(
        resp_edit_own.status().is_success(),
        "status is {}",
        resp_edit_own.status()
    );

    // edit other submission
    let edit_req_other = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{}", submission_id).to_string())
        .insert_header(("Authorization", format!("Bearer {}", token_2)))
        .set_json(&submission_edit_json)
        .to_request();

    let resp_edit_other = test::call_service(&app, edit_req_other).await;
    assert!(
        resp_edit_other.status().is_client_error(),
        "status is {}",
        resp_edit_other.status()
    );

    // edit other submission as mod
    let edit_req_mod = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{}", submission_id).to_string())
        .insert_header(("Authorization", format!("Bearer {}", token_mod)))
        .set_json(&submission_edit_json)
        .to_request();

    let resp_edit_mod = test::call_service(&app, edit_req_mod).await;
    assert!(
        resp_edit_mod.status().is_success(),
        "status is {}",
        resp_edit_mod.status()
    );
}

#[actix_web::test]
async fn submission_aredlplus_boost() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let (user_id_2, _) = create_test_user(&db, None).await;
    let (user_id_mod, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;

    let role_id: i32 = diesel::insert_into(roles::table)
        .values((
            roles::privilege_level.eq(5),
            roles::role_desc.eq(format!("Test Role - AREDL+")),
        ))
        .returning(roles::id)
        .get_result(&mut db.connection().unwrap())
        .expect("Failed to create test role");

    diesel::insert_into(user_roles::table)
        .values((
            user_roles::role_id.eq(role_id),
            user_roles::user_id.eq(user_id_2),
        ))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to assign role to user");

    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let token2 =
        create_test_token(user_id_2, &auth.jwt_encoding_key).expect("Failed to generate token");
    let token_mod =
        create_test_token(user_id_mod, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;

    // video_url
    let submission_data = json!({
        "level_id": level_id,
        "video_url": "https://youtube.com/watch?v=xvFZjo5PgG0",
        "raw_url": "https://youtube.com/watch?v=xvFZjo5PgG0",
        "mobile": false
    });

    let req = test::TestRequest::post()
        .uri("/aredl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&submission_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "First submission failed: status {}",
        resp.status()
    );
    let resp_body = test::read_body(resp).await;
    let submission1: serde_json::Value =
        serde_json::from_slice(&resp_body).expect("Failed to parse response body");

    let req = test::TestRequest::post()
        .uri("/aredl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", token2)))
        .set_json(&submission_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "Second submission failed: {}",
        resp.status()
    );
    let resp_body = test::read_body(resp).await;
    let submission2: serde_json::Value =
        serde_json::from_slice(&resp_body).expect("Failed to parse response body");

    assert_eq!(
        submission1["priority"].as_bool().unwrap(),
        false,
        "Priority field for user 1 is not false as expected"
    );
    assert_eq!(
        submission2["priority"].as_bool().unwrap(),
        true,
        "Priority field for user 2 is not true as expected"
    );

    let claim_req = test::TestRequest::get()
        .uri("/aredl/submissions/claim")
        .insert_header(("Authorization", format!("Bearer {}", token_mod)))
        .to_request();

    let claim_resp = test::call_service(&app, claim_req).await;
    assert!(
        claim_resp.status().is_success(),
        "Claim request failed: {}",
        claim_resp.status()
    );
    let body: serde_json::Value = test::read_body_json(claim_resp).await;
    assert_eq!(body["id"], submission2["id"])
}

#[actix_web::test]
async fn submission_banned_player() {
    let (app, db, auth, _) = init_test_app().await;

    let (not_banned, _) = create_test_user(&db, None).await;
    let not_banned_token =
        create_test_token(not_banned, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;

    let (banned, _) = create_test_user(&db, None).await;

    diesel::update(users::table)
        .filter(users::id.eq(banned))
        .set(users::ban_level.eq(2))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to ban user!");

    let banned_token =
        create_test_token(banned, &auth.jwt_encoding_key).expect("Failed to generate token");

    let submission_data = json!({
        "level_id": level_id,
        "video_url": "https://youtube.com/watch?v=xvFZjo5PgG0",
        "raw_url": "https://youtube.com/watch?v=xvFZjo5PgG0",
        "mobile": false,
    });

    let req_1 = test::TestRequest::post()
        .uri("/aredl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", not_banned_token)))
        .set_json(&submission_data)
        .to_request();

    let resp_1 = test::call_service(&app, req_1).await;
    assert!(
        resp_1.status().is_success(),
        "status of req 1 is {}",
        resp_1.status()
    );

    let req_2 = test::TestRequest::post()
        .uri("/aredl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", banned_token)))
        .set_json(&submission_data)
        .to_request();

    let resp_2 = test::call_service(&app, req_2).await;
    assert!(
        resp_2.status().is_client_error(),
        "status of req 2 is {}",
        resp_2.status()
    )
}

#[actix_web::test]
async fn delete_submission() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;

    let submission: Uuid = create_test_submission(level_id, user_id, &db).await;

    let req = test::TestRequest::delete()
        .uri(format!("/aredl/submissions/{submission}").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "status of req is {}",
        resp.status()
    );
}

#[actix_web::test]
async fn get_global_queue() {
    let (app, db, _, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let level = create_test_level(&db).await;
    create_test_submission(level, user, &db).await;

    let req = test::TestRequest::get()
        .uri("/aredl/submissions/queue")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["submissions_in_queue"].as_i64().unwrap(), 1);
    assert_eq!(body["uc_submissions"].as_i64().unwrap(), 0);
}

#[actix_web::test]
async fn get_submission_queue() {
    let (app, db, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&db).await;
    let submission = create_test_submission(level, user, &db).await;

    let req = test::TestRequest::get()
        .uri(&format!("/aredl/submissions/{submission}/queue"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["total"].as_i64().unwrap(), 1);
}

#[actix_web::test]
async fn patch_submission_no_changes() {
    let (app, db, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&db).await;
    let submission = create_test_submission(level, user, &db).await;

    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn patch_submission_invalid_urls() {
    let (app, db, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&db).await;
    let submission = create_test_submission(level, user, &db).await;

    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"video_url":"not a url"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"raw_url":"not a url"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn patch_submission_mod_errors() {
    let (app, db, auth, _) = init_test_app().await;
    let (moderator, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(moderator, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&db).await;
    let submission = create_test_submission(level, moderator, &db).await;

    // no changes
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // duplicate submission
    let dup_level = create_test_level(&db).await;
    let _other = create_test_submission(dup_level, moderator, &db).await;
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"level_id": dup_level}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn banned_player_resubmission() {
    let (app, db, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&db).await;
    let submission = create_test_submission(level, user, &db).await;

    diesel::update(submissions::table)
        .filter(submissions::id.eq(submission))
        .set(submissions::status.eq(SubmissionStatus::Denied))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    diesel::update(users::table)
        .filter(users::id.eq(user))
        .set(users::ban_level.eq(2))
        .execute(&mut db.connection().unwrap())
        .unwrap();

    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn accept_submission() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let (moderator_id, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token =
        create_test_token(moderator_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;

    let submission: Uuid = create_test_submission(level_id, user_id, &db).await;

    let accept_data = json!({"status": "Accepted", "reviewer_notes": "GG!"});

    let req = test::TestRequest::patch()
        .uri(format!("/aredl/submissions/{submission}").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&accept_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "status of req is {}",
        resp.status()
    );

    records::table
        .filter(records::level_id.eq(level_id))
        .filter(records::submitted_by.eq(user_id))
        .select(records::id)
        .first::<Uuid>(&mut db.connection().unwrap())
        .expect("Failed to get new record!");

    let accepted_submission = submissions::table
        .filter(submissions::id.eq(submission))
        .select(Submission::as_select())
        .first::<Submission>(&mut db.connection().unwrap())
        .expect("Failed to get accepted submission!");

    assert_eq!(
        accepted_submission.status,
        SubmissionStatus::Accepted,
        "Submission status is not Accepted!"
    );

    assert_eq!(
        accepted_submission.reviewer_notes.unwrap(),
        accept_data["reviewer_notes"].as_str().unwrap(),
        "Reviewer notes do not match!"
    );

    let history_entry = submission_history::table
        .filter(submission_history::submission_id.eq(submission))
        .order(submission_history::timestamp.desc())
        .select(SubmissionHistory::as_select())
        .first::<SubmissionHistory>(&mut db.connection().unwrap())
        .expect("Failed to get submission history!");

    assert_eq!(
        history_entry.status,
        SubmissionStatus::Accepted,
        "Submission history status is not Accepted!"
    );

    assert_eq!(
        history_entry.reviewer_notes.unwrap(),
        accept_data["reviewer_notes"].as_str().unwrap(),
        "Submission history reviewer notes do not match!"
    );
}

#[actix_web::test]
async fn deny_submission() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let (moderator_id, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token =
        create_test_token(moderator_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;

    let submission: Uuid = create_test_submission(level_id, user_id, &db).await;

    let deny_data = json!({"status": "Denied", "reviewer_notes": "No Cheat Indicator:tm:"});

    let req = test::TestRequest::patch()
        .uri(format!("/aredl/submissions/{submission}").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&deny_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "status of req is {}",
        resp.status()
    );

    let denied_submission = submissions::table
        .filter(submissions::id.eq(submission))
        .select(Submission::as_select())
        .first::<Submission>(&mut db.connection().unwrap())
        .expect("Failed to get denied submission!");
    assert_eq!(
        denied_submission.status,
        SubmissionStatus::Denied,
        "Submission status is not Denied!"
    );

    assert_eq!(
        denied_submission.reviewer_notes.unwrap(),
        deny_data["reviewer_notes"].as_str().unwrap(),
        "Reviewer notes do not match!"
    );

    let history_entry = submission_history::table
        .filter(submission_history::submission_id.eq(submission))
        .order(submission_history::timestamp.desc())
        .select(SubmissionHistory::as_select())
        .first::<SubmissionHistory>(&mut db.connection().unwrap())
        .expect("Failed to get submission history!");

    assert_eq!(
        history_entry.status,
        SubmissionStatus::Denied,
        "Submission history status is not Denied!"
    );

    assert_eq!(
        history_entry.reviewer_notes.unwrap(),
        deny_data["reviewer_notes"].as_str().unwrap(),
        "Submission history reviewer notes do not match!"
    );
}

#[actix_web::test]
async fn submission_under_consideration() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let (moderator_id, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token =
        create_test_token(moderator_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;

    let submission: Uuid = create_test_submission(level_id, user_id, &db).await;

    let under_consideration_data = json!({"status": "UnderConsideration", "reviewer_notes": "No way SpaceUK is hacking right guys"});

    let req = test::TestRequest::patch()
        .uri(format!("/aredl/submissions/{submission}").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&under_consideration_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(
        resp.status().is_success(),
        "status of req is {}",
        resp.status()
    );

    let uc_submission = submissions::table
        .filter(submissions::id.eq(submission))
        .select(Submission::as_select())
        .first::<Submission>(&mut db.connection().unwrap())
        .expect("Failed to get UC submission!");

    assert_eq!(
        uc_submission.status,
        SubmissionStatus::UnderConsideration,
        "Submission status is not UnderConsideration!"
    );

    assert_eq!(
        uc_submission.reviewer_notes.unwrap(),
        under_consideration_data["reviewer_notes"].as_str().unwrap(),
        "Reviewer notes do not match!"
    );

    let history_entry = submission_history::table
        .filter(submission_history::submission_id.eq(submission))
        .order(submission_history::timestamp.desc())
        .select(SubmissionHistory::as_select())
        .first::<SubmissionHistory>(&mut db.connection().unwrap())
        .expect("Failed to get submission history!");

    assert_eq!(
        history_entry.status,
        SubmissionStatus::UnderConsideration,
        "Submission history status is not UnderConsideration!"
    );

    assert_eq!(
        history_entry.reviewer_notes.unwrap(),
        under_consideration_data["reviewer_notes"].as_str().unwrap(),
        "Submission history reviewer notes do not match!"
    );
}

#[actix_web::test]
async fn increment_shift() {
    let (app, db, auth, _) = init_test_app().await;
    let (submitter_id, _) = create_test_user(&db, None).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token_mod = create_test_token(mod_id, &auth.jwt_encoding_key).unwrap();
    let shift_id = create_test_shift(&db, mod_id, true).await;
    let level = create_test_level(&db).await;
    create_test_submission(level, submitter_id, &db).await;

    let req = test::TestRequest::get()
        .uri("/aredl/submissions/claim")
        .insert_header(("Authorization", format!("Bearer {}", token_mod)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    let sub_id = body["id"].as_str().unwrap().to_string();

    let accept_data = json!({"status": "Accepted", "reviewer_notes":"ok"});
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{sub_id}"))
        .insert_header(("Authorization", format!("Bearer {}", token_mod)))
        .set_json(&accept_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let count: i32 = shifts::table
        .find(shift_id)
        .select(shifts::completed_count)
        .first(&mut db.connection().unwrap())
        .unwrap();
    assert_eq!(count, 1);
}

#[actix_web::test]
async fn shift_completes() {
    use diesel::{ExpressionMethods, RunQueryDsl};

    let (app, db, auth, _) = init_test_app().await;
    let (submitter_id, _) = create_test_user(&db, None).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(mod_id, &auth.jwt_encoding_key).unwrap();
    let shift_id = create_test_shift(&db, mod_id, true).await;
    diesel::update(shifts::table.filter(shifts::id.eq(shift_id)))
        .set(shifts::target_count.eq(1))
        .execute(&mut db.connection().unwrap())
        .unwrap();
    let level = create_test_level(&db).await;
    create_test_submission(level, submitter_id, &db).await;

    let req = test::TestRequest::get()
        .uri("/aredl/submissions/claim")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = read_body_json(resp).await;
    let sub_id = body["id"].as_str().unwrap();

    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{sub_id}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"status": "Accepted"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let status: ShiftStatus = shifts::table
        .find(shift_id)
        .select(shifts::status)
        .first(&mut db.connection().unwrap())
        .unwrap();
    assert_eq!(status, ShiftStatus::Completed);
}

#[actix_web::test]
async fn reviewer_submission_can_set_reviewer_fields_for_other_users() {
    let (app, db, auth, _) = init_test_app().await;

    let (reviewer_id, _) = create_test_user(&db, Some(Permission::SubmissionReview)).await;
    let (other_user_id, _) = create_test_user(&db, None).await;
    let reviewer_token =
        create_test_token(reviewer_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let other_level = create_test_level(&db).await;
    let reviewer_level = create_test_level(&db).await;

    let other_submission = json!({
        "submitted_by": other_user_id,
        "level_id": other_level,
        "video_url": "https://www.youtube.com/watch?v=other111111",
        "raw_url": "https://www.youtube.com/watch?v=otherraw111",
        "mobile": false,
        "status": "UnderConsideration",
        "reviewer_notes": "Initial review notes",
    });

    let other_req = test::TestRequest::post()
        .uri("/aredl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", reviewer_token)))
        .set_json(&other_submission)
        .to_request();

    let other_resp = test::call_service(&app, other_req).await;
    assert!(
        other_resp.status().is_success(),
        "status is {}",
        other_resp.status()
    );
    let other_body: serde_json::Value = test::read_body_json(other_resp).await;

    let other_submission_id = Uuid::parse_str(other_body["id"].as_str().unwrap())
        .expect("Response missing submission id");

    let stored_other_submission = submissions::table
        .find(other_submission_id)
        .select(Submission::as_select())
        .first::<Submission>(&mut db.connection().unwrap())
        .expect("Failed to fetch stored submission");

    assert_eq!(
        stored_other_submission.status,
        SubmissionStatus::UnderConsideration,
        "Reviewer provided status should be applied for other users",
    );
    assert_eq!(
        stored_other_submission.reviewer_notes.as_deref(),
        Some("Initial review notes"),
        "Reviewer notes should be stored for other users",
    );

    let reviewer_submission = json!({
        "level_id": reviewer_level,
        "video_url": "https://www.youtube.com/watch?v=self1111111",
        "raw_url": "https://www.youtube.com/watch?v=selfraw1111",
        "mobile": false,
        "status": "Accepted",
        "reviewer_notes": "Should not be applied",
    });

    let reviewer_req = test::TestRequest::post()
        .uri("/aredl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", reviewer_token)))
        .set_json(&reviewer_submission)
        .to_request();

    let reviewer_resp = test::call_service(&app, reviewer_req).await;
    assert!(
        reviewer_resp.status().is_success(),
        "status is {}",
        reviewer_resp.status()
    );
    let reviewer_body: serde_json::Value = test::read_body_json(reviewer_resp).await;

    let reviewer_submission_id = Uuid::parse_str(reviewer_body["id"].as_str().unwrap())
        .expect("Response missing reviewer submission id");

    let stored_reviewer_submission = submissions::table
        .find(reviewer_submission_id)
        .select(Submission::as_select())
        .first::<Submission>(&mut db.connection().unwrap())
        .expect("Failed to fetch reviewer submission");

    assert_eq!(
        stored_reviewer_submission.status,
        SubmissionStatus::Pending,
        "Reviewer status should be ignored on own submissions",
    );
    assert!(
        stored_reviewer_submission.reviewer_notes.is_none(),
        "Reviewer notes should be ignored on own submissions",
    );
}
