#[cfg(test)]
use crate::{
    aredl::{levels::test_utils::create_test_level_with_record, records::Record},
    auth::{create_test_token, Permission},
    schema::{aredl::records, merge_requests},
    test_utils::*,
    users::{
        merge::test_utils::create_test_merge_req,
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
async fn merge_req() {
    let (app, mut conn, auth) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&mut conn, Some(Permission::MergeReview)).await;
    let (user_2_id, _) = create_test_placeholder_user(&mut conn, None).await;
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
    let body: serde_json::Value = test::read_body_json(resp).await;
    println!("{:?}", body);
    // assert!(resp.status().is_success(), "status is {}", resp.status());

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
async fn accept_merge_req() {
    let (app, mut conn, auth) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&mut conn, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&mut conn, None).await;
    let (mod_id, _) = create_test_user(&mut conn, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let (_, user_1_record_id) = create_test_level_with_record(&mut conn, user_1_id).await;
    let (_, user_2_record_id) = create_test_level_with_record(&mut conn, user_2_id).await;

    let merge = create_test_merge_req(&mut conn, user_1_id, user_2_id).await;

    let req = test::TestRequest::post()
        .uri(format!("/users/merge/requests/{merge}/accept").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());

    let records = records::table
        .filter(records::submitted_by.eq(user_1_id))
        .select(Record::as_select())
        .get_results::<Record>(&mut conn)
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
    .get_result::<bool>(&mut conn)
    .expect("Failed to check for merge!");

    assert_ne!(merge_exists, true, "Merge request exists!")
}

#[actix_web::test]
async fn direct_merge() {
    let (app, mut conn, auth) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&mut conn, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&mut conn, None).await;
    let (mod_id, _) = create_test_user(&mut conn, Some(Permission::DirectMerge)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let (_, user_1_record_id) = create_test_level_with_record(&mut conn, user_1_id).await;
    let (_, user_2_record_id) = create_test_level_with_record(&mut conn, user_2_id).await;

    let merge_data = json!({
        "primary_user": user_1_id.to_string(),
        "secondary_user": user_2_id.to_string()
    });

    let req = test::TestRequest::post()
        .uri(format!("/users/merge/").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&merge_data)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());

    let records = records::table
        .filter(records::submitted_by.eq(user_1_id))
        .select(Record::as_select())
        .get_results::<Record>(&mut conn)
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
}

#[actix_web::test]
async fn reject_merge() {
    let (app, mut conn, auth) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&mut conn, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&mut conn, None).await;
    let (mod_id, _) = create_test_user(&mut conn, Some(Permission::DirectMerge)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    create_test_level_with_record(&mut conn, user_1_id).await;
    create_test_level_with_record(&mut conn, user_2_id).await;

    let merge = create_test_merge_req(&mut conn, user_1_id, user_2_id).await;

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
        .get_result::<i64>(&mut conn)
        .expect("Failed to get records!");

    assert_eq!(records, 1, "User does not have exactly 2 records!");

    assert_eq!(
        body["is_rejected"].as_bool().unwrap(),
        true,
        "Request is not marked as rejected!"
    )
}
