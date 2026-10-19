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
    uuid::Uuid,
};

#[actix_web::test]
async fn set_level_tags() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::LevelTagsModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;
    let tag_id = create_test_tag(&db).await;

    let req = test::TestRequest::post()
        .uri(&format!("/classic/levels/{level_id}/tags"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!([tag_id, tag_id]))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body, json!([tag_id]));

    let req = test::TestRequest::post()
        .uri(&format!("/classic/levels/{level_id}/tags"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!([Uuid::new_v4()]))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response!(resp, StatusCode::UNPROCESSABLE_ENTITY, None);
    assert_eq!(get_test_level(&db, level_id).await.tags, vec![tag_id]);

    let req = test::TestRequest::post()
        .uri(&format!("/classic/levels/{level_id}/tags"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!([]))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body, json!([]));
    assert!(get_test_level(&db, level_id).await.tags.is_empty());
}

#[actix_web::test]
async fn add_level_tags() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::LevelTagsModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;
    let tag_id = create_test_tag(&db).await;

    let req = test::TestRequest::patch()
        .uri(&format!("/classic/levels/{level_id}/tags"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!([tag_id, tag_id]))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = read_body_json(resp).await;

    assert_eq!(body, json!([tag_id]));
    assert_eq!(get_test_level(&db, level_id).await.tags, vec![tag_id]);
}

#[actix_web::test]
async fn delete_level_tags() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::LevelTagsModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let level_id = create_test_level(&db).await;
    let tag_id = create_test_tag(&db).await;
    create_test_level_tag(&db, level_id, tag_id).await;

    let req = test::TestRequest::delete()
        .uri(&format!("/classic/levels/{level_id}/tags"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!([tag_id]))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = read_body_json(resp).await;

    assert_eq!(body, json!([]));
    assert!(get_test_level(&db, level_id).await.tags.is_empty());
}

#[actix_web::test]
async fn list_level_tags() {
    let (app, db, _, _) = init_test_app().await;
    let level_id = create_test_level(&db).await;
    let tag_id = create_test_tag(&db).await;
    create_test_level_tag(&db, level_id, tag_id).await;

    let req = test::TestRequest::get()
        .uri(&format!("/classic/levels/{level_id}/tags"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let tags: serde_json::Value = read_body_json(resp).await;
    assert_eq!(
        tags,
        json!([{
            "id": tag_id,
            "list_id": "classic",
            "name": "Wave",
            "description": "Extreme demons that feature mostly wave gameplay.",
            "category": "Gamemode",
        }])
    );

    let req = test::TestRequest::get().uri("/classic/levels").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body[0]["tags"], json!([tag_id]));

    let req = test::TestRequest::get()
        .uri(&format!("/classic/levels/{level_id}"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), StatusCode::OK);
    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["tags"], tags);
}
