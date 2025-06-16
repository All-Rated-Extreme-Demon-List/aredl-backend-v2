#[cfg(test)]
use crate::{
    aredl::{levels::test_utils::create_test_level_with_record, records::Record},
    auth::{create_test_token, Permission},
    schema::aredl::records,
    test_utils::*,
    users::{
        merge::test_utils::create_test_merge_log,
        test_utils::{create_test_placeholder_user, create_test_user},
    },
};
#[cfg(test)]
use actix_web::test::read_body_json;
#[cfg(test)]
use actix_web::{self, test};
#[cfg(test)]
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
#[cfg(test)]
use serde_json::json;

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
async fn list_merge_logs() {
    let (app, mut conn, auth) = init_test_app().await;

    let (user_1_id, _) = create_test_user(&mut conn, None).await;
    let (user_2_id, _) = create_test_placeholder_user(&mut conn, None).await;
    let (mod_id, _) = create_test_user(&mut conn, Some(Permission::MergeReview)).await;
    let token =
        create_test_token(mod_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let log_id = create_test_merge_log(&mut conn, user_1_id, user_2_id).await;

    let req = test::TestRequest::get()
        .uri("/users/merge/logs")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    assert!(body["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|l| l["id"] == log_id.to_string()));
}
