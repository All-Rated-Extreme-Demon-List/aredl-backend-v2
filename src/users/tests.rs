#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission},
    test_utils::init_test_app,
    users::test_utils::create_test_user,
};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn create_placeholder_user() {
    let (app, mut conn, auth) = init_test_app().await;

    let (staff_user_id, _) = create_test_user(&mut conn, Some(Permission::PlaceholderCreate)).await;
    let staff_token =
        create_test_token(staff_user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let placeholder_payload = json!({
        "username": "test_placeholder"
    });

    let req = test::TestRequest::post()
        .uri("/users/placeholders")
        .insert_header(("Authorization", format!("Bearer {}", staff_token)))
        .set_json(&placeholder_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let created_user: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(created_user["username"], "test_placeholder");
}

#[actix_web::test]
async fn update_user_info() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::UserModify)).await;
    let (staff_user_id, _) = create_test_user(&mut conn, Some(Permission::UserBan)).await;
    let staff_token =
        create_test_token(staff_user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let update_payload = json!({
        "global_name": "Updated Name",
        "description": "Updated description"
    });

    let req = test::TestRequest::patch()
        .uri(&format!("/users/{}", user_id))
        .insert_header(("Authorization", format!("Bearer {}", staff_token)))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let updated_user: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(updated_user["global_name"], "Updated Name");
    assert_eq!(updated_user["description"], "Updated description");
}

#[actix_web::test]
async fn update_user_info_less_privilege() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::UserModify)).await;
    let user_token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
    let (staff_user_id, _) = create_test_user(&mut conn, Some(Permission::UserBan)).await;

    let update_payload = json!({
        "global_name": "Updated Name",
        "description": "Updated description"
    });

    let req = test::TestRequest::patch()
        .uri(&format!("/users/{}", staff_user_id))
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 403);
}

#[actix_web::test]
async fn ban_user() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, username) = create_test_user(&mut conn, None).await;
    let (staff_user_id, _) = create_test_user(&mut conn, Some(Permission::UserBan)).await;
    let staff_token =
        create_test_token(staff_user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let ban_payload = json!({ "ban_level": 2 });

    let req = test::TestRequest::patch()
        .uri(&format!("/users/{}/ban", user_id))
        .insert_header(("Authorization", format!("Bearer {}", staff_token)))
        .set_json(&ban_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let banned_user: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(banned_user["ban_level"], 2);

    let req = test::TestRequest::get()
        .uri(&format!("/users?name_filter=%{}%", username))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let users: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(users["data"].as_array().unwrap()[0]["ban_level"], 2);
}

#[actix_web::test]
async fn find_user() {
    let (app, mut conn, _auth) = init_test_app().await;
    let (user_id, username) = create_test_user(&mut conn, None).await;

    let req = test::TestRequest::get()
        .uri(&format!("/users/{}", user_id))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let user: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(user["username"], username);
}

#[actix_web::test]
async fn list_users() {
    let (app, mut conn, _auth) = init_test_app().await;
    let (_, username) = create_test_user(&mut conn, None).await;
    create_test_user(&mut conn, None).await;

    let req = test::TestRequest::get().uri("/users").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let users: serde_json::Value = test::read_body_json(resp).await;
    assert!(users["data"].as_array().unwrap().len() >= 2);

    let req = test::TestRequest::get()
        .uri(&format!("/users?name_filter=%{}%", username))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let users: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(users["data"].as_array().unwrap().len(), 1);
    assert_eq!(
        users["data"].as_array().unwrap()[0]["global_name"],
        username
    );
}

#[actix_web::test]
async fn user_character_limit() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::UserModify)).await;
    let (staff_user_id, _) = create_test_user(&mut conn, Some(Permission::UserBan)).await;
    let staff_token =
        create_test_token(staff_user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let update_payload = json!({
        "global_name": "This is a 35 characters or longer username that should return an error",
    });

    let req = test::TestRequest::patch()
        .uri(&format!("/users/{}", user_id))
        .insert_header(("Authorization", format!("Bearer {}", staff_token)))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}
