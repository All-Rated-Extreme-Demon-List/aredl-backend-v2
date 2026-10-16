#[cfg(test)]
use {
    crate::{
        auth::{create_test_token, Permission},
        list::{
            levels::custom_copies::test_utils::create_test_custom_copy,
            levels::test_utils::create_test_level, List,
        },
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
        "id_types": ["Bugfix", "Ldm"],
        "status": "Allowed",
    });
    let req = test::TestRequest::post()
        .uri(format!("/classic/levels/{level_id}/custom-copies").as_str())
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&custom_copy_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
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
    assert_eq!(body["id_types"], custom_copy_data["id_types"]);
    assert_eq!(body["added_by"], user_id.to_string());
    assert_eq!(body["published"], false);
}

#[actix_web::test]
async fn update_custom_copy() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::LevelCustomCopiesModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;

    let custom_copy = create_test_custom_copy(&db, List::Classic, level_id, user_id).await;

    let custom_copy_data = json!({
        "status": "Banned",
        "id_types": ["StartPos", "Ldm"],
        "published": true,
    });
    let req = test::TestRequest::patch()
        .uri(format!("/classic/levels/{level_id}/custom-copies/{custom_copy}").as_str())
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&custom_copy_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["status"], custom_copy_data["status"]);
    assert_eq!(body["id_types"], custom_copy_data["id_types"]);
    assert_eq!(body["published"], true);

    let req = test::TestRequest::patch()
        .uri(&format!(
            "/classic/levels/{level_id}/custom-copies/{custom_copy}"
        ))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"status": "Allowed"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["status"], "Allowed");
    assert_eq!(body["published"], true);
    assert_eq!(body["id_types"], custom_copy_data["id_types"]);

    let req = test::TestRequest::patch()
        .uri(&format!(
            "/classic/levels/{level_id}/custom-copies/{custom_copy}"
        ))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"published": false}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["status"], "Allowed");
    assert_eq!(body["published"], false);
}

#[actix_web::test]
async fn delete_custom_copy() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::LevelCustomCopiesModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;

    let custom_copy = create_test_custom_copy(&db, List::Classic, level_id, user_id).await;

    let req = test::TestRequest::delete()
        .uri(format!("/classic/levels/{level_id}/custom-copies/{custom_copy}").as_str())
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::NO_CONTENT);
    assert!(test::read_body(resp).await.is_empty());
}

#[actix_web::test]
async fn list_custom_copies() {
    let (app, db, _, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::LevelCustomCopiesModify)).await;
    let level_id = create_test_level(&db).await;

    create_test_custom_copy(&db, List::Classic, level_id, user_id).await;
    create_test_custom_copy(&db, List::Classic, level_id, user_id).await;

    let req = test::TestRequest::get()
        .uri(format!("/classic/levels/{level_id}/custom-copies?type_filter=Bugfix&status_filter=Allowed&description=%es%").as_str())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let data = body.as_array().unwrap();

    assert_eq!(data.len(), 2);
    assert!(data
        .iter()
        .all(|x| x["added_by"]["id"] == user_id.to_string()));
    assert!(data.iter().all(|x| x["published"] == false));

    let req = test::TestRequest::get()
        .uri(&format!(
            "/classic/levels/{level_id}/custom-copies?published_filter=true"
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let copies: Vec<serde_json::Value> = read_body_json(resp).await;
    assert!(copies.is_empty());

    let req = test::TestRequest::get()
        .uri(&format!(
            "/classic/levels/{level_id}/custom-copies?published_filter=false"
        ))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let copies: Vec<serde_json::Value> = read_body_json(resp).await;
    assert_eq!(copies.len(), 2);
}

#[actix_web::test]
async fn custom_copy_id_is_unique_per_level_on_create_and_update() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::LevelCustomCopiesModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;
    let other_level_id = create_test_level(&db).await;

    let req = test::TestRequest::post()
        .uri(&format!("/classic/levels/{level_id}/custom-copies"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"copy_id": 123_456, "id_types": ["Bugfix"], "status": "Allowed"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);

    for target_level in [level_id, other_level_id] {
        let req = test::TestRequest::post()
            .uri(&format!("/classic/levels/{target_level}/custom-copies"))
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(json!({"copy_id": 123_456, "id_types": ["Ldm"], "status": "Banned"}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        if target_level == level_id {
            assert_error_response!(resp, StatusCode::CONFLICT, None);
        } else {
            assert_eq!(resp.status(), StatusCode::CREATED);
        }
    }

    let req = test::TestRequest::post()
        .uri(&format!("/classic/levels/{other_level_id}/custom-copies"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"copy_id": 654_321, "id_types": ["Bugfix"], "status": "Allowed"}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = read_body_json(resp).await;
    let copy_uuid = body["id"].as_str().expect("Missing custom copy ID");

    let req = test::TestRequest::patch()
        .uri(&format!(
            "/classic/levels/{other_level_id}/custom-copies/{copy_uuid}"
        ))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"copy_id": 123_456}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response!(resp, StatusCode::CONFLICT, None);
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
        "id_types": ["Bugfix"],
        "status": "Allowed"
    });
    let req = test::TestRequest::post()
        .uri(format!("/classic/levels/{level_id}/custom-copies").as_str())
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
