#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper},
    schema::users,
    test_utils::init_test_app,
    users::test_utils::create_test_user,
};
use crate::{
    test_utils::init_test_db_state,
    users::{User, UserUpsert},
};
#[cfg(test)]
use actix_web::{test, web};
#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn create_placeholder_user() {
    let (app, mut conn, auth, _) = init_test_app().await;

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
    assert_eq!(created_user["global_name"], "test_placeholder");
}

#[actix_web::test]
async fn update_user_info() {
    let (app, mut conn, auth, _) = init_test_app().await;
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
    let (app, mut conn, auth, _) = init_test_app().await;
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
    let (app, mut conn, auth, _) = init_test_app().await;
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
    let (app, mut conn, _, _) = init_test_app().await;
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
    let (app, mut conn, _, _) = init_test_app().await;
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
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::UserModify)).await;
    let (staff_user_id, _) = create_test_user(&mut conn, Some(Permission::UserBan)).await;
    let user_token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");
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
    assert!(resp.status().is_success());

    let req = test::TestRequest::post()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}

#[actix_web::test]
async fn list_users_with_filters() {
    let (app, mut conn, _, _) = init_test_app().await;
    let (_, name) = create_test_user(&mut conn, None).await;
    let (placeholder_id, _) =
        crate::users::test_utils::create_test_placeholder_user(&mut conn, None).await;

    let req = test::TestRequest::get()
        .uri("/users?placeholder=true")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let users: serde_json::Value = test::read_body_json(resp).await;
    assert!(users["data"]
        .as_array()
        .unwrap()
        .iter()
        .any(|u| u["id"] == placeholder_id.to_string()));

    let req = test::TestRequest::get()
        .uri(&format!("/users?name_filter=%{}%&per_page=1&page=1", name))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let users: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(users["data"].as_array().unwrap().len(), 1);
}

#[actix_web::test]
async fn upsert_creates_and_updates_user() {
    let (_, _, _, _) = init_test_app().await;
    let db_state = init_test_db_state();
    let db_data = web::Data::new(db_state.clone());

    let user_upsert = UserUpsert {
        username: "new_user".to_string(),
        global_name: "New User".to_string(),
        discord_id: Some("123".to_string()),
        placeholder: false,
        country: Some(1),
        discord_avatar: Some("avatar".to_string()),
        discord_banner: None,
        discord_accent_color: None,
    };

    let created = User::upsert(db_data.clone(), user_upsert).expect("insert");
    assert_eq!(created.username, "new_user");
    assert_eq!(created.discord_id.as_deref(), Some("123"));

    let mut conn = db_state.connection().unwrap();
    let fetched = users::table
        .filter(users::id.eq(created.id))
        .select(User::as_select())
        .first::<User>(&mut conn)
        .unwrap();
    assert_eq!(fetched.username, "new_user");

    let update_upsert = UserUpsert {
        username: "updated".to_string(),
        global_name: "Updated".to_string(),
        discord_id: Some("123".to_string()),
        placeholder: false,
        country: Some(2),
        discord_avatar: Some("newavatar".to_string()),
        discord_banner: Some("banner".to_string()),
        discord_accent_color: Some(5),
    };

    let updated = User::upsert(db_data.clone(), update_upsert).expect("update");
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.username, "updated");
    assert_eq!(updated.country, Some(1));
    assert_eq!(updated.global_name, "New User");
}

#[actix_web::test]
async fn placeholder_random_username() {
    let (app, mut conn, auth, _) = init_test_app().await;

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
    assert_ne!(created_user["username"], "test_placeholder");
}