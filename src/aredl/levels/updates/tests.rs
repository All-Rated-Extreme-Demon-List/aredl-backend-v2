use actix_http::StatusCode;
#[cfg(test)]
use {
    crate::{
        aredl::{
            levels::test_utils::create_test_level, levels::updates::test_utils::create_test_update,
        },
        auth::{create_test_token, Permission},
        test_utils::{assert_error_response, init_test_app},
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
    serde_json::json,
};

#[actix_web::test]
async fn create_update() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;
    let timestamp = chrono::Utc::now();

    let update_data = json!({
        "changelog": "buffed the last ship",
        "update_type": "Buff",
        "timestamp": timestamp,
    });
    let req = test::TestRequest::post()
        .uri(format!("/aredl/levels/updates/{level_id}").as_str())
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    assert_eq!(
        level_id.to_string(),
        body["level_id"],
        "Level IDs do not match!"
    );
    assert_eq!(body["changelog"], update_data["changelog"]);
    assert_eq!(body["update_type"], update_data["update_type"]);
    assert!(body["timestamp"].is_string());
}

#[actix_web::test]
async fn update_update() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;
    let update_id = create_test_update(&db, level_id).await;
    let timestamp = chrono::Utc::now();

    let update_data = json!({
        "changelog": "balanced transitions",
        "update_type": "Balance",
        "timestamp": timestamp,
    });
    let req = test::TestRequest::patch()
        .uri(format!("/aredl/levels/updates/{update_id}").as_str())
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body["changelog"], update_data["changelog"]);
    assert_eq!(body["update_type"], update_data["update_type"]);
}

#[actix_web::test]
async fn delete_update() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, Some(Permission::LevelModify)).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;
    let update_id = create_test_update(&db, level_id).await;

    let req = test::TestRequest::delete()
        .uri(format!("/aredl/levels/updates/{update_id}").as_str())
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}

#[actix_web::test]
async fn list_updates() {
    let (app, db, _, _) = init_test_app().await;

    let level_id = create_test_level(&db).await;

    create_test_update(&db, level_id).await;
    create_test_update(&db, level_id).await;

    let req = test::TestRequest::get()
        .uri(format!("/aredl/levels/updates?level_id={level_id}&type_filter=Buff").as_str())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());

    let body: serde_json::Value = read_body_json(resp).await;
    let data = body["data"].as_array().unwrap();

    assert_eq!(data.len(), 2);
    assert!(data.iter().all(|x| x["level_id"] == level_id.to_string()));
    assert!(data.iter().all(|x| x["update_type"] == "Buff"));
}

#[actix_web::test]
async fn create_update_requires_level_modify() {
    let (app, db, auth, _) = init_test_app().await;

    let (user_id, _) = create_test_user(&db, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_id = create_test_level(&db).await;

    let update_data = json!({
        "changelog": "test update",
        "update_type": "Other",
        "timestamp": chrono::Utc::now()
    });
    let req = test::TestRequest::post()
        .uri(format!("/aredl/levels/updates/{level_id}").as_str())
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response!(
        resp,
        StatusCode::FORBIDDEN,
        Some("You do not have the required permission (level_modify) to access this endpoint"),
    );
}
