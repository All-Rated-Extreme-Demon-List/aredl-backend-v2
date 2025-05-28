#[cfg(test)]
use crate::test_utils::*;
#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission},
    schema::{merge_requests, aredl::records},
    db::DbConnection,
    aredl::records::{Record, tests::create_test_record}

};
use actix_web::test::read_body_json;
#[cfg(test)]
use diesel::{ExpressionMethods, RunQueryDsl, SelectableHelper, QueryDsl, dsl::exists, select};
#[cfg(test)]
use uuid::Uuid;
#[cfg(test)]
use actix_web::{self, test};
#[cfg(test)]
use serde_json::json;

#[cfg(test)]
async fn create_test_merge_req(conn: &mut DbConnection, user_1: Uuid, user_2: Uuid) -> Uuid {
    diesel::insert_into(merge_requests::table)
        .values((
            // this becomes the new user
            merge_requests::primary_user.eq(user_1),
            merge_requests::secondary_user.eq(user_2),
            merge_requests::is_rejected.eq(false),
            merge_requests::is_claimed.eq(false),
        ))
        .returning(merge_requests::id)
        .get_result::<Uuid>(conn)
        .expect("Failed to create test merge request!")
}

#[actix_web::test]
async fn merge_req() {
    let (app, mut conn, auth) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, Some(Permission::MergeReview)).await;
    let (user_2, _) = create_test_placeholder_user(&mut conn, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");


    let req_data = json!({
        "secondary_user": user_2.to_string()
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
    

    assert_eq!(body["secondary_user"].as_str().unwrap().to_string(), user_2.to_string(), "Secondary users do not match!");
    assert_eq!(body["primary_user"].as_str().unwrap().to_string(), user_id.to_string(), "Primary users do not match!");
    assert_eq!(body["is_rejected"].as_bool().unwrap(), false, "Request is rejected!");
    assert_eq!(body["is_claimed"].as_bool().unwrap(), false, "Request is claimed!");
}

#[actix_web::test]
async fn accept_merge_req() {
    let (app, mut conn, auth) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, None).await;
    let (user_2, _) = create_test_placeholder_user(&mut conn, None).await;
    let (mod_id, _) = create_test_user(&mut conn, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let user_1_record = create_test_record(&mut conn, user_id).await;
    let user_2_record = create_test_record(&mut conn, user_2).await;

    let merge = create_test_merge_req(&mut conn, user_id, user_2).await;

    let req = test::TestRequest::post()
        .uri(format!("/users/merge/requests/{merge}/accept").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());
    
    let records = records::table
        .filter(records::submitted_by.eq(user_id))
        .select(Record::as_select())
        .get_results::<Record>(&mut conn)
        .expect("Failed to collect records!");
    
    assert_eq!(
        records.len(), 2, "User does not have exactly 2 records!"
    );
    assert!(
        records.iter().any(|x| x.id == user_1_record.id), "Did not return first record!"
    );
    assert!(
        records.iter().any(|x| x.id == user_2_record.id), "Did not return second record!"
    );

    let merge_exists = select(
        exists(
            merge_requests::table.filter(
                merge_requests::id.eq(merge)
            )
        )
    )
    .get_result::<bool>(&mut conn)
    .expect("Failed to check for merge!");

    assert_ne!(merge_exists, true, "Merge request exists!")
}

#[actix_web::test]
async fn direct_merge() {
    let (app, mut conn, auth) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, None).await;
    let (user_2, _) = create_test_placeholder_user(&mut conn, None).await;
    let (mod_id, _) = create_test_user(&mut conn, Some(Permission::DirectMerge)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let user_1_record = create_test_record(&mut conn, user_id).await;
    let user_2_record = create_test_record(&mut conn, user_2).await;

    let merge_data = json!({
        "primary_user": user_id.to_string(),
        "secondary_user": user_2.to_string()
    });

    let req = test::TestRequest::post()
        .uri(format!("/users/merge/").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&merge_data)
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());
    
    let records = records::table
        .filter(records::submitted_by.eq(user_id))
        .select(Record::as_select())
        .get_results::<Record>(&mut conn)
        .expect("Failed to collect records!");
    
    assert_eq!(
        records.len(), 2, "User does not have exactly 2 records!"
    );
    assert!(
        records.iter().any(|x| x.id == user_1_record.id), "Did not return first record!"
    );
    assert!(
        records.iter().any(|x| x.id == user_2_record.id), "Did not return second record!"
    );
}

#[actix_web::test]
async fn reject_merge() {
    let (app, mut conn, auth) = init_test_app().await;

    let (user_id, _) = create_test_user(&mut conn, None).await;
    let (user_2, _) = create_test_placeholder_user(&mut conn, None).await;
    let (mod_id, _) = create_test_user(&mut conn, Some(Permission::DirectMerge)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    create_test_record(&mut conn, user_id).await;
    create_test_record(&mut conn, user_2).await;

    let merge = create_test_merge_req(&mut conn, user_id, user_2).await;

    let req = test::TestRequest::post()
        .uri(format!("/users/merge/requests/{merge}/reject").as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());
    let body: serde_json::Value = read_body_json(res).await;

    let records = records::table
        .filter(records::submitted_by.eq(user_id))
        .count()
        .get_result::<i64>(&mut conn)
        .expect("Failed to get records!");
    
    assert_eq!(
        records, 1, "User does not have exactly 2 records!"
    );

    assert_eq!(body["is_rejected"].as_bool().unwrap(), true, "Request is not marked as rejected!")
    
}
