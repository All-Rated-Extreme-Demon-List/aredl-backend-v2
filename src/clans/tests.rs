#[cfg(test)]
use crate::{
    auth::create_test_token, schema::clan_members, test_utils::init_test_app,
    users::test_utils::create_test_user,
};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use serde_json::json;
use uuid::Uuid;

#[actix_web::test]
async fn create_and_join() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, None).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let payload = json!({"global_name": "Test Clan", "tag": "TC"});
    let req = test::TestRequest::post()
        .uri("/clans")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let clan: serde_json::Value = test::read_body_json(resp).await;
    let clan_id = Uuid::parse_str(clan["id"].as_str().unwrap()).unwrap();
    let count: i64 = clan_members::table
        .filter(clan_members::clan_id.eq(clan_id))
        .filter(clan_members::user_id.eq(user_id))
        .count()
        .get_result(&mut conn)
        .unwrap();
    assert_eq!(count, 1);
}

#[actix_web::test]
async fn list_clans() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, None).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let payload = json!({"global_name": "List Clan", "tag": "LC"});
    let req = test::TestRequest::post()
        .uri("/clans")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&payload)
        .to_request();
    let _resp = test::call_service(&app, req).await;

    let req = test::TestRequest::get().uri("/clans").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body["data"].as_array().unwrap().len() >= 1);
}
