use crate::aredl::levels::ldms::test_utils::create_test_ldm;
#[cfg(test)]
use crate::{
    aredl::{
        levels::test_utils::create_test_level
    },
    auth::{create_test_token, Permission},
    
};
#[cfg(test)]
use crate::{test_utils::*, users::test_utils::create_test_user};
#[cfg(test)]
use actix_web::test::{self, read_body_json};
#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn create_ldm() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&mut conn).await;

    let ldm_data = json!({
        "ldm_id": 123456,
        "description": "Bugfix",
    });
    let req = test::TestRequest::post()
        .uri(format!("/aredl/levels/ldms/{}", level_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&ldm_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;
    
    assert_eq!(
        level_id.to_string(),
        body["level_id"],
        "Level IDs do not match!"
    );
    assert_eq!(
        ldm_data["ldm_id"],
        body["ldm_id"].as_i64().unwrap(),
        "Level IDs do not match!"
    );
    assert_eq!(
        body["is_allowed"],
        true
    );
    assert_eq!(
        body["added_by"],
        user_id.to_string()
    );
}

#[actix_web::test]
async fn update_ldm() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&mut conn).await;

    let ldm = create_test_ldm(&mut conn, level_id, user_id).await;

    let ldm_data = json!({
        "is_allowed": false
    });
    let req = test::TestRequest::patch()
        .uri(format!("/aredl/levels/ldms/{}", ldm).as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&ldm_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        body["is_allowed"],
        ldm_data["is_allowed"]
    );
}

#[actix_web::test]
async fn delete_ldm() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&mut conn).await;

    let ldm = create_test_ldm(&mut conn, level_id, user_id).await;

    let req = test::TestRequest::delete()
        .uri(format!("/aredl/levels/ldms/{}", ldm).as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn list_ldms() {
    let (app, mut conn, _, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::LevelModify)).await;
    let level_id = create_test_level(&mut conn).await;

    create_test_ldm(&mut conn, level_id, user_id).await;
    create_test_ldm(&mut conn, level_id, user_id).await;

    let req = test::TestRequest::get()
        .uri(format!("/aredl/levels/ldms?level_id={}&is_allowed=true&description=%es%", level_id).as_str())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();

    assert_eq!(
        data.len(),
        2
    );
    assert!(
        data.iter()
            .all(
                |x| x["added_by"]["id"] == user_id.to_string()
            )
    );
}

#[actix_web::test]
async fn ldm_auth() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&mut conn).await;

    let ldm_data = json!({
        "ldm_id": 123456,
        "description": "Bugfix"
    });
    let req = test::TestRequest::post()
        .uri(format!("/aredl/levels/ldms/{}", level_id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&ldm_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error(), "status is {}", resp.status());
}