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
#[cfg(test)]
use serde_json::json;
#[cfg(test)]
use uuid::Uuid;

#[actix_web::test]
async fn create_submission() {
    let (app, mut conn, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;

    let submission_data = json!({
        "level_id": level_id,
        "video_url": "https://video.com",
        "raw_url": "https://raw.com"
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
    let (app, mut conn, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;

    let submission_data = json!({
        "level_id": level_id,
        "video_url": "https://video.com"
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
    let (app, mut conn, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;

    // video_url
    let submission_data = json!({
        "level_id": level_id,
        "video_url": "slkdfjskdlf",
        "raw_url": "https://raw.com"
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
        "video_url": "https://video.com",
        "raw_url": "isldjfsdkf"
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
    let (app, mut conn, auth, _) = init_test_app().await;

    let (user_id_1, _) = create_test_user(&mut conn, None).await;
    let token_1 =
        create_test_token(user_id_1, &auth.jwt_encoding_key).expect("Failed to generate token");

    let (user_id_2, _) = create_test_user(&mut conn, None).await;
    let token_2 =
        create_test_token(user_id_2, &auth.jwt_encoding_key).expect("Failed to generate token");

    let (user_id_mod, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token_mod =
        create_test_token(user_id_mod, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&mut conn).await;

    let submission_id = create_test_submission(level_id, user_id_1, &mut conn).await;

    let submission_edit_json = json!({
        "video_url": "https://new_video.com"
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
    let (app, mut conn, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, None).await;
    let (user_id_2, _) = create_test_user(&mut conn, None).await;
    let (user_id_mod, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;

    let role_id: i32 = diesel::insert_into(roles::table)
        .values((
            roles::privilege_level.eq(5),
            roles::role_desc.eq(format!("Test Role - AREDL+")),
        ))
        .returning(roles::id)
        .get_result(&mut conn)
        .expect("Failed to create test role");

    diesel::insert_into(user_roles::table)
        .values((
            user_roles::role_id.eq(role_id),
            user_roles::user_id.eq(user_id_2),
        ))
        .execute(&mut conn)
        .expect("Failed to assign role to user");

    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let token2 =
        create_test_token(user_id_2, &auth.jwt_encoding_key).expect("Failed to generate token");
    let token_mod =
        create_test_token(user_id_mod, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;

    // video_url
    let submission_data = json!({
        "level_id": level_id,
        "video_url": "https://video.com",
        "raw_url": "https://raw.com"
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
    let (app, mut conn, auth, _) = init_test_app().await;

    let (not_banned, _) = create_test_user(&mut conn, None).await;
    let not_banned_token =
        create_test_token(not_banned, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;

    let (banned, _) = create_test_user(&mut conn, None).await;

    diesel::update(users::table)
        .filter(users::id.eq(banned))
        .set(users::ban_level.eq(2))
        .execute(&mut conn)
        .expect("Failed to ban user!");

    let banned_token =
        create_test_token(banned, &auth.jwt_encoding_key).expect("Failed to generate token");

    let submission_data = json!({
        "level_id": level_id,
        "video_url": "https://video.com",
        "raw_url": "https://raw.com"
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
    let (app, mut conn, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;

    let submission: Uuid = create_test_submission(level_id, user_id, &mut conn).await;

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
async fn get_submission_history() {
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

    let res = test::call_service(&app, req).await;
    assert!(
        res.status().is_success(),
        "status of req is {}",
        res.status()
    );

    let req = test::TestRequest::get()
        .uri(format!("/aredl/submissions/{submission}/history").as_str())
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
async fn get_global_queue() {
    let (app, mut conn, _, _) = init_test_app().await;
    let (user, _) = create_test_user(&mut conn, None).await;
    let level = create_test_level(&mut conn).await;
    create_test_submission(level, user, &mut conn).await;

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
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&mut conn, None).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&mut conn).await;
    let submission = create_test_submission(level, user, &mut conn).await;

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
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&mut conn, None).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&mut conn).await;
    let submission = create_test_submission(level, user, &mut conn).await;

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
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&mut conn, None).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&mut conn).await;
    let submission = create_test_submission(level, user, &mut conn).await;

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
async fn patch_submission_level_errors() {
    use crate::schema::aredl::{levels, submissions};
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

    let (app, mut conn, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&mut conn, None).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&mut conn).await;
    let submission = create_test_submission(level, user, &mut conn).await;

    // level not found
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"level_id": Uuid::new_v4()}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // legacy level
    let legacy_level = create_test_level(&mut conn).await;
    diesel::update(levels::table.filter(levels::id.eq(legacy_level)))
        .set((levels::legacy.eq(true), levels::position.eq(2)))
        .execute(&mut conn)
        .unwrap();

    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"level_id": legacy_level}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // top400 requires raw footage
    diesel::update(submissions::table.filter(submissions::id.eq(submission)))
        .set(submissions::raw_url.eq::<Option<String>>(None))
        .execute(&mut conn)
        .unwrap();
    let top_level = create_test_level(&mut conn).await;
    diesel::update(levels::table.filter(levels::id.eq(top_level)))
        .set(levels::position.eq(1))
        .execute(&mut conn)
        .unwrap();
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"level_id": top_level}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // duplicate submission
    let dup_level = create_test_level(&mut conn).await;
    let _other = create_test_submission(dup_level, user, &mut conn).await;
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"level_id": dup_level}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn patch_submission_mod_errors() {
    use crate::schema::users;
    use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

    let (app, mut conn, auth, _) = init_test_app().await;
    let (moderator, _) = create_test_user(&mut conn, Some(Permission::SubmissionReview)).await;
    let token = create_test_token(moderator, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&mut conn).await;
    let submission = create_test_submission(level, moderator, &mut conn).await;

    // no changes
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // user not found
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"submitted_by": Uuid::new_v4()}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // banned user
    let (banned, _) = create_test_user(&mut conn, None).await;
    diesel::update(users::table.filter(users::id.eq(banned)))
        .set(users::ban_level.eq(2))
        .execute(&mut conn)
        .unwrap();
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"submitted_by": banned}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // level not found
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"level_id": Uuid::new_v4()}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());

    // duplicate submission
    let dup_level = create_test_level(&mut conn).await;
    let _other = create_test_submission(dup_level, moderator, &mut conn).await;
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
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user, _) = create_test_user(&mut conn, None).await;
    let token = create_test_token(user, &auth.jwt_encoding_key).unwrap();
    let level = create_test_level(&mut conn).await;
    let submission = create_test_submission(level, user, &mut conn).await;

    diesel::update(submissions::table)
        .filter(submissions::id.eq(submission))
        .set(submissions::status.eq(SubmissionStatus::Denied))
        .execute(&mut conn)
        .unwrap();

    diesel::update(users::table)
        .filter(users::id.eq(user))
        .set(users::ban_level.eq(2))
        .execute(&mut conn)
        .unwrap();

    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/submissions/{submission}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}
