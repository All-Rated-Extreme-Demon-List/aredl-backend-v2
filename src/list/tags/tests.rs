#[cfg(test)]
use {
    crate::{
        auth::{create_test_token, Permission},
        list::{
            levels::test_utils::{create_test_level, get_test_level},
            tags::test_utils::{create_test_level_tag, create_test_tag},
        },
        test_utils::{assert_error_response, init_test_app},
        users::test_utils::create_test_user,
    },
    actix_http::StatusCode,
    actix_web::test::{self, read_body_json},
    serde_json::json,
};

#[actix_web::test]
async fn create_tag() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::TagModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let tag_data = json!({
        "name": "2.2",
        "description": "Extreme demons released in update 2.2.",
        "category": "Version",
    });
    let req = test::TestRequest::post()
        .uri("/classic/tags")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&tag_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::CREATED);
    let body: serde_json::Value = read_body_json(resp).await;

    assert_eq!(body["name"], tag_data["name"]);
    assert_eq!(body["description"], tag_data["description"]);
    assert_eq!(body["category"], tag_data["category"]);
    assert_eq!(body["list_id"], "classic");
}

#[actix_web::test]
async fn update_tag() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::TagModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let tag_id = create_test_tag(&db).await;

    let tag_data = json!({
        "name": "Clicksync",
        "description": "Extreme demons with gameplay synced to the music.",
        "category": "Gameplay",
    });
    let req = test::TestRequest::patch()
        .uri(&format!("/classic/tags/{tag_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&tag_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = read_body_json(resp).await;

    assert_eq!(body["id"], tag_id.to_string());
    assert_eq!(body["name"], tag_data["name"]);
    assert_eq!(body["description"], tag_data["description"]);
    assert_eq!(body["category"], tag_data["category"]);
}

#[actix_web::test]
async fn delete_tag() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::TagModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;
    let tag_id = create_test_tag(&db).await;
    create_test_level_tag(&db, level_id, tag_id).await;

    let req = test::TestRequest::delete()
        .uri(&format!("/classic/tags/{tag_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = read_body_json(resp).await;

    assert_eq!(body["id"], tag_id.to_string());
    assert!(get_test_level(&db, level_id).await.tags.is_empty());
}

#[actix_web::test]
async fn list_tags() {
    let (app, db, _, _) = init_test_app().await;
    let tag_id = create_test_tag(&db).await;

    let req = test::TestRequest::get().uri("/classic/tags").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = read_body_json(resp).await;

    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["id"], tag_id.to_string());
    assert_eq!(body[0]["name"], "Wave");
    assert_eq!(
        body[0]["description"],
        "Extreme demons that feature mostly wave gameplay."
    );
    assert_eq!(body[0]["category"], "Gamemode");
}

#[actix_web::test]
async fn tag_validation() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::TagModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    create_test_tag(&db).await;

    let tag_data = json!({
        "name": "Wave",
        "description": "Extreme demons that feature mostly wave gameplay.",
        "category": "Gamemode",
    });
    let req = test::TestRequest::post()
        .uri("/classic/tags")
        .set_json(&tag_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response!(
        resp,
        StatusCode::UNAUTHORIZED,
        Some("You must be authenticated to access this endpoint")
    );

    let req = test::TestRequest::post()
        .uri("/classic/tags")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&tag_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response!(resp, StatusCode::CONFLICT, None);

    let req = test::TestRequest::post()
        .uri("/classic/tags")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({
            "name": " ",
            "description": "Extreme demons that feature mostly wave gameplay.",
            "category": "Gamemode",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response!(resp, StatusCode::UNPROCESSABLE_ENTITY, None);
}
