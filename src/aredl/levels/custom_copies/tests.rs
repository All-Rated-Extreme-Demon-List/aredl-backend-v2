#[cfg(test)]
use {
    crate::{
        aredl::{
            levels::custom_copies::test_utils::create_test_custom_copy,
            levels::test_utils::create_test_level,
        },
        auth::{create_test_token, Permission},
        test_utils::{assert_error_response, init_test_app},
        users::test_utils::create_test_user,
    },
    actix_http::StatusCode,
    actix_web::test::{self, read_body_json},
    serde_json::json,
};

#[actix_web::test]
async fn create_custom_copy() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::LevelCustomCopiesModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;

    let custom_copy_data = json!({
        "copy_id": 123_456,
        "id_type": "Bugfix",
        "status": "Allowed",
    });
    let req = test::TestRequest::post()
        .uri(format!("/aredl/levels/{level_id}/custom-copies").as_str())
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&custom_copy_data)
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
        custom_copy_data["copy_id"],
        body["copy_id"].as_i64().unwrap(),
        "Level IDs do not match!"
    );
    assert_eq!(custom_copy_data["id_type"], "Bugfix");
    assert_eq!(body["added_by"], user_id.to_string());
}

#[actix_web::test]
async fn update_custom_copy() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::LevelCustomCopiesModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;

    let custom_copy = create_test_custom_copy(&db, level_id, user_id).await;

    let custom_copy_data = json!({
        "status": "Banned",
        "id_type": "Ldm"
    });
    let req = test::TestRequest::patch()
        .uri(format!("/aredl/levels/{level_id}/custom-copies/{custom_copy}").as_str())
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&custom_copy_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["status"], custom_copy_data["status"]);
    assert_eq!(body["id_type"], custom_copy_data["id_type"]);
}

#[actix_web::test]
async fn delete_custom_copy() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::LevelCustomCopiesModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;

    let custom_copy = create_test_custom_copy(&db, level_id, user_id).await;

    let req = test::TestRequest::delete()
        .uri(format!("/aredl/levels/{level_id}/custom-copies/{custom_copy}").as_str())
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn list_custom_copies() {
    let (app, db, _, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::LevelCustomCopiesModify)).await;
    let level_id = create_test_level(&db).await;

    create_test_custom_copy(&db, level_id, user_id).await;
    create_test_custom_copy(&db, level_id, user_id).await;

    let req = test::TestRequest::get()
        .uri(format!("/aredl/levels/{level_id}/custom-copies?type_filter=Bugfix&status_filter=Allowed&description=%es%").as_str())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let data = body.as_array().unwrap();

    assert_eq!(data.len(), 2);
    assert!(data
        .iter()
        .all(|x| x["added_by"]["id"] == user_id.to_string()));
}

#[actix_web::test]
async fn create_custom_copy_requires_level_custom_copies_modify() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;

    let custom_copy_data = json!({
        "copy_id": 123_456,
        "description": "test description",
        "id_type": "Bugfix",
        "status": "Allowed"
    });
    let req = test::TestRequest::post()
        .uri(format!("/aredl/levels/{level_id}/custom-copies").as_str())
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&custom_copy_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response!(
        resp,
        StatusCode::FORBIDDEN,
        Some("You do not have the required permission (level_custom_copies_modify) to access this endpoint"),
    );
}
