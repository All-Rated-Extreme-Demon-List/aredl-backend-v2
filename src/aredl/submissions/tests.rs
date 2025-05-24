#[cfg(test)]
use crate::test_utils::*;
#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission},
    schema::{roles, user_roles, users},
};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use diesel::{ExpressionMethods, RunQueryDsl};
#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn create_submission() {
    let (app, mut conn, auth) = init_test_app().await;

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
    assert_eq!(body["submitted_by"].as_str().unwrap().to_string(), user_id.to_string(), "Submitters do not match!")
}

#[actix_web::test]
async fn submission_without_raw() {
    let (app, mut conn, auth) = init_test_app().await;

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
    let (app, mut conn, auth) = init_test_app().await;

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
    let (app, mut conn, auth) = init_test_app().await;

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

    let submission_data = json!({
        "level_id": level_id,
        "video_url": "https://video.com",
        "raw_url": "https://raw.com"
    });

    let submission_req = test::TestRequest::post()
        .uri("/aredl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", token_1)))
        .set_json(&submission_data)
        .to_request();

    let resp = test::call_service(&app, submission_req).await;
    assert!(
        resp.status().is_success(),
        "initial submission request status is {}",
        resp.status()
    );

    let resp_body = test::read_body(resp).await;
    let submission: serde_json::Value =
        serde_json::from_slice(&resp_body).expect("Failed to parse response body");
    let submission_id = submission["id"].as_str().expect("Submission ID not found");

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
    let (app, mut conn, auth) = init_test_app().await;

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
    let (app, mut conn, auth) = init_test_app().await;

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
    assert!(resp_1.status().is_success(), "status of req 1 is {}", resp_1.status());

    let req_2 = test::TestRequest::post()
        .uri("/aredl/submissions")
        .insert_header(("Authorization", format!("Bearer {}", banned_token)))
        .set_json(&submission_data)
        .to_request();

    let resp_2 = test::call_service(&app, req_2).await;
    assert!(resp_2.status().is_client_error(), "status of req 2 is {}", resp_2.status())
}
