#[cfg(test)]
use crate::test_utils::*;
#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission}
};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn list_levels() {
    let (app, _, _) = init_test_app().await;
    let req = test::TestRequest::get().uri("/aredl/levels").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn update_level() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::LevelModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&mut conn).await;
    let update_data = json!({
        "name": "Updated Level Name"
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/aredl/levels/{}", level_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn find_level() {
    let (app, mut conn, _auth) = init_test_app().await;
    let level_id = create_test_level(&mut conn).await;
    let req = test::TestRequest::get().uri(&format!("/aredl/levels/{}", level_id)).to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}
