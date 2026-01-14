#[cfg(test)]
use crate::{
    aredl::{levels::test_utils::create_test_level_with_record, records::Record},
    auth::{create_test_token, Permission},
    schema::{aredl::records, merge_requests},
    test_utils::*,
    users::{
        merge::requests::test_utils::create_test_merge_req,
        test_utils::{create_test_placeholder_user, create_test_user},
    },
};
#[cfg(test)]
use actix_web::test::read_body_json;
#[cfg(test)]
use actix_web::{self, test};
#[cfg(test)]
use diesel::{dsl::exists, select, ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn create_merge_request() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&db, Some(Permission::MergeReview)).await;
    let (user_2_id, _) = create_test_placeholder_user(&db, None).await;
    let token =
        create_test_token(user_1_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let req_data = json!({
        "secondary_user": user_2_id.to_string()
    });

    let req = test::TestRequest::post()
        .uri("/users/merge/requests")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&req_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(
        body["secondary_user"].as_str().unwrap().to_string(),
        user_2_id.to_string(),
        "Secondary users do not match!"
    );
    assert_eq!(
        body["primary_user"].as_str().unwrap().to_string(),
        user_1_id.to_string(),
        "Primary users do not match!"
    );
    assert_eq!(
        body["is_rejected"].as_bool().unwrap(),
        false,
        "Request is rejected!"
    );
    assert_eq!(
        body["is_claimed"].as_bool().unwrap(),
        false,
        "Request is claimed!"
    );
}

#[actix_web::test]
async fn accept_merge_request() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&db, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&db, None).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let (_, user_1_record_id) = create_test_level_with_record(&db, user_1_id).await;
    let (_, user_2_record_id) = create_test_level_with_record(&db, user_2_id).await;

    let merge = create_test_merge_req(&db, user_1_id, user_2_id).await;

    let req = test::TestRequest::post()
        .uri(format!("/users/merge/requests/{merge}/accept").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());

    let records = records::table
        .filter(records::submitted_by.eq(user_1_id))
        .select(Record::as_select())
        .get_results::<Record>(&mut db.connection().unwrap())
        .expect("Failed to collect records!");

    assert_eq!(records.len(), 2, "User does not have exactly 2 records!");
    assert!(
        records.iter().any(|x| x.id == user_1_record_id),
        "Did not return first record!"
    );
    assert!(
        records.iter().any(|x| x.id == user_2_record_id),
        "Did not return second record!"
    );

    let merge_exists = select(exists(
        merge_requests::table.filter(merge_requests::id.eq(merge)),
    ))
    .get_result::<bool>(&mut db.connection().unwrap())
    .expect("Failed to check for merge!");

    assert_ne!(merge_exists, true, "Merge request exists!")
}

#[actix_web::test]
async fn reject_merge_request() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&db, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&db, None).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::DirectMerge)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    create_test_level_with_record(&db, user_1_id).await;
    create_test_level_with_record(&db, user_2_id).await;

    let merge = create_test_merge_req(&db, user_1_id, user_2_id).await;

    let req = test::TestRequest::post()
        .uri(format!("/users/merge/requests/{merge}/reject").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());
    let body: serde_json::Value = read_body_json(res).await;

    let records = records::table
        .filter(records::submitted_by.eq(user_1_id))
        .count()
        .get_result::<i64>(&mut db.connection().unwrap())
        .expect("Failed to get records!");

    assert_eq!(records, 1, "User does not have exactly 2 records!");

    assert_eq!(
        body["is_rejected"].as_bool().unwrap(),
        true,
        "Request is not marked as rejected!"
    )
}

#[actix_web::test]
async fn create_merge_request_rejects_self_merge() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let req_data = json!({
        "secondary_user": user_id.to_string()
    });

    let req = test::TestRequest::post()
        .uri("/users/merge/requests")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&req_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "status is {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["message"].as_str().unwrap(),
        "You cannot merge your account with itself."
    );
}

#[actix_web::test]
async fn create_merge_request_rejects_unknown_user() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let req_data = json!({
        "secondary_user": uuid::Uuid::new_v4().to_string()
    });

    let req = test::TestRequest::post()
        .uri("/users/merge/requests")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&req_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404, "status is {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["message"].as_str().unwrap(),
        "The secondary user does not exist."
    );
}

#[actix_web::test]
async fn create_merge_request_rejects_non_placeholder_user() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::MergeReview)).await;
    let (secondary_id, _) = create_test_user(&db, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let req_data = json!({
        "secondary_user": secondary_id.to_string()
    });

    let req = test::TestRequest::post()
        .uri("/users/merge/requests")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&req_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400, "status is {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["message"].as_str().unwrap(),
        "You can only submit merge requests for placeholder users. To merge your account with a user that is already linked to another discord account, please make a support post on our discord server."
    );
}

#[actix_web::test]
async fn create_merge_request_rejects_duplicate_submission() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&db, Some(Permission::MergeReview)).await;
    let (user_2_id, _) = create_test_placeholder_user(&db, None).await;
    let token =
        create_test_token(user_1_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    create_test_merge_req(&db, user_1_id, user_2_id).await;

    let req_data = json!({
        "secondary_user": user_2_id.to_string()
    });

    let req = test::TestRequest::post()
        .uri("/users/merge/requests")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&req_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409, "status is {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(
        body["message"].as_str().unwrap(),
        "You already submitted a merge request for your account. Please wait until it's either accepted or denied before submitting a new one."
    );
}

#[actix_web::test]
async fn list_merge_requests() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&db, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&db, None).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let merge = create_test_merge_req(&db, user_1_id, user_2_id).await;

    let req = test::TestRequest::get()
        .uri("/users/merge/requests")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r["id"] == merge.to_string()));
}

#[actix_web::test]
async fn find_merge_request() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&db, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&db, None).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let merge = create_test_merge_req(&db, user_1_id, user_2_id).await;

    let req = test::TestRequest::get()
        .uri(&format!("/users/merge/requests/{merge}"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["id"].as_str().unwrap(), merge.to_string());
}

#[actix_web::test]
async fn list_merge_requests_filter_is_claimed() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&db, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&db, None).await;
    let (user_3_id, _) = create_test_user(&db, None).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let claimed_merge = create_test_merge_req(&db, user_1_id, user_2_id).await;
    let unclaimed_merge = create_test_merge_req(&db, user_3_id, user_2_id).await;

    diesel::update(merge_requests::table)
        .filter(merge_requests::id.eq(claimed_merge))
        .set(merge_requests::is_claimed.eq(true))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to claim merge request");

    let req = test::TestRequest::get()
        .uri("/users/merge/requests?claimed_filter=true")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let ids: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_str().unwrap().to_string())
        .collect();
    assert!(ids.contains(&claimed_merge.to_string()));
    assert!(!ids.contains(&unclaimed_merge.to_string()));
}

#[actix_web::test]
async fn list_merge_requests_filter_is_rejected() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&db, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&db, None).await;
    let (user_3_id, _) = create_test_user(&db, None).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let rejected_merge = create_test_merge_req(&db, user_1_id, user_2_id).await;
    let pending_merge = create_test_merge_req(&db, user_3_id, user_2_id).await;

    diesel::update(merge_requests::table)
        .filter(merge_requests::id.eq(rejected_merge))
        .set(merge_requests::is_rejected.eq(true))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to reject merge request");

    let req = test::TestRequest::get()
        .uri("/users/merge/requests?rejected_filter=true")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let ids: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_str().unwrap().to_string())
        .collect();
    assert!(ids.contains(&rejected_merge.to_string()));
    assert!(!ids.contains(&pending_merge.to_string()));
}

#[actix_web::test]
async fn list_merge_requests_filter_user() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_1_id, user_1_name) = create_test_user(&db, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&db, None).await;
    let (user_3_id, _) = create_test_user(&db, None).await;
    let (user_4_id, _) = create_test_placeholder_user(&db, None).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let matching_merge = create_test_merge_req(&db, user_1_id, user_2_id).await;
    let other_merge = create_test_merge_req(&db, user_3_id, user_4_id).await;

    let req = test::TestRequest::get()
        .uri(&format!(
            "/users/merge/requests?user_filter={}",
            user_1_name
        ))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let ids: Vec<String> = body["data"]
        .as_array()
        .unwrap()
        .iter()
        .map(|row| row["id"].as_str().unwrap().to_string())
        .collect();
    assert!(ids.contains(&matching_merge.to_string()));
    assert!(!ids.contains(&other_merge.to_string()));
}

#[actix_web::test]
async fn claim_merge_request() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&db, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&db, None).await;
    let (user_3_id, _) = create_test_user(&db, None).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let merge_1 = create_test_merge_req(&db, user_1_id, user_2_id).await;
    create_test_merge_req(&db, user_3_id, user_2_id).await;

    let req = test::TestRequest::get()
        .uri("/users/merge/requests/claim")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert!(
        body["id"].as_str().unwrap() == merge_1.to_string(),
        "Claimed merge request does not match oldest request ID!"
    );
}

#[actix_web::test]
async fn claim_merge_request_when_none_exist() {
    let (app, _db, auth, _) = init_test_app().await;

    let (mod_id, _) = create_test_user(&_db, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let req = test::TestRequest::get()
        .uri("/users/merge/requests/claim")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body.is_null(), "Expected null body when no requests");
}

#[actix_web::test]
async fn unclaim_merge_request() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&db, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&db, None).await;
    let (mod_id, _) = create_test_user(&db, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let merge = create_test_merge_req(&db, user_1_id, user_2_id).await;

    diesel::update(merge_requests::table)
        .filter(merge_requests::id.eq(merge))
        .set(merge_requests::is_claimed.eq(true))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to claim merge request");

    let req = test::TestRequest::post()
        .uri(&format!("/users/merge/requests/{merge}/unclaim"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let claimed: bool = merge_requests::table
        .filter(merge_requests::id.eq(merge))
        .select(merge_requests::is_claimed)
        .first(&mut db.connection().unwrap())
        .expect("Failed to fetch merge request");
    assert!(!claimed, "Request should be unclaimed");
}
