#[cfg(test)]
use crate::test_utils::*;
#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission}
};
#[cfg(test)]
use actix_web::test;
use actix_web::test::read_body_json;
#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn create_level() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::LevelModify)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_data = json!({
        "name": "Test Level",
        "position": 1,
        "level_id": 123456,
        "publisher_id": user_id.to_string(),
        "legacy": false,
        "two_player": false
    });
    let req = test::TestRequest::post()
        .uri("/aredl/levels")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&level_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(level_data["level_id"].as_i64().unwrap(), body["level_id"].as_i64().unwrap(), "Level IDs do not match!")
}

#[actix_web::test]
async fn list_levels() {
    let (app, mut conn, _) = init_test_app().await;
    let req = test::TestRequest::get().uri("/aredl/levels").to_request();
    let resp = test::call_service(&app, req).await;
    create_test_level(&mut conn).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_ne!(body.as_array().unwrap().len(), 0, "Response is empty!");
    assert_eq!(body[0].as_object().unwrap()["position"].as_i64().unwrap(), 1, "First level returned is not the top 1!")
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
    
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["name"].to_string(), update_data["name"].to_string())
}

#[actix_web::test]
async fn find_level() {
    let (app, mut conn, _auth) = init_test_app().await;
    let level_id = create_test_level(&mut conn).await;
    let req = test::TestRequest::get().uri(&format!("/aredl/levels/{}", level_id)).to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(level_id.to_string(), body["id"].as_str().unwrap().to_string(), "IDs do not match!")
}
