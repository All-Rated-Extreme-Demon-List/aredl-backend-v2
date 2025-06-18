#[cfg(test)]
use crate::{
    aredl::levels::test_utils::{
        create_test_level as create_test_aredl_level,
        create_test_level_with_record as create_test_aredl_level_with_record,
    },
    arepl::levels::test_utils::create_test_level_with_record as create_test_arepl_level_with_record,
    auth::create_test_token,
    schema::{aredl, arepl, users},
    test_utils::init_test_app,
    users::test_utils::create_test_user,
};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn get_authenticated_user() {
    let (app, mut conn, auth, _) = init_test_app().await;

    let (user_id, username) = create_test_user(&mut conn, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let req = test::TestRequest::get()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let user: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(user["username"], username);
}

#[actix_web::test]
async fn update_authenticated_user() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, None).await;
    let user_token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let update_payload = json!({
        "global_name": "Updated Name",
        "description": "Updated description",
        "ban_level": 1,
        "country": 10
    });

    let req = test::TestRequest::patch()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let updated_user: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(updated_user["global_name"], "Updated Name");
    assert_eq!(updated_user["description"], "Updated description");
    assert_eq!(updated_user["ban_level"], 1);
    assert_eq!(updated_user["country"], 10);

    let req = test::TestRequest::get()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let user: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(user["global_name"], "Updated Name");
    assert_eq!(user["description"], "Updated description");
    assert_eq!(user["ban_level"], 1);
    assert_eq!(user["country"], 10);
}

#[actix_web::test]
async fn update_authenticated_user_banned() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, None).await;
    let user_token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    diesel::update(users::table.filter(users::id.eq(user_id)))
        .set(users::ban_level.eq(2))
        .execute(&mut conn)
        .expect("Failed to ban user");

    let update_payload = json!({
        "ban_level": 1
    });

    let req = test::TestRequest::patch()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 403);
}

#[actix_web::test]
async fn update_authenticated_user_country_cooldown() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, None).await;
    let user_token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    diesel::update(users::table.filter(users::id.eq(user_id)))
        .set(users::last_country_update.eq(chrono::Utc::now()))
        .execute(&mut conn)
        .expect("Failed to update last country update");

    let update_payload = json!({
        "country": 10
    });

    let req = test::TestRequest::patch()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {}", user_token)))
        .set_json(&update_payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400);
}

#[actix_web::test]
async fn update_background_level_aredl() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let (level_uuid, _) = create_test_aredl_level_with_record(&mut conn, user_id).await;

    let level_id = aredl::levels::table
        .filter(aredl::levels::id.eq(level_uuid))
        .select(aredl::levels::level_id)
        .first::<i32>(&mut conn)
        .expect("Failed to get level ID");

    let req = test::TestRequest::patch()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({ "background_level": level_id }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let updated: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(updated["background_level"], level_id);
}

#[actix_web::test]
async fn update_background_level_arepl() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let (level_uuid, _) = create_test_arepl_level_with_record(&mut conn, user_id).await;

    let level_id = arepl::levels::table
        .filter(arepl::levels::id.eq(level_uuid))
        .select(arepl::levels::level_id)
        .first::<i32>(&mut conn)
        .expect("Failed to get level ID");

    let req = test::TestRequest::patch()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({ "background_level": level_id }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let updated: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(updated["background_level"], level_id);
}

#[actix_web::test]
async fn update_background_level_not_beaten() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let level_uuid = create_test_aredl_level(&mut conn).await;

    let level_id = aredl::levels::table
        .filter(aredl::levels::id.eq(level_uuid))
        .select(aredl::levels::level_id)
        .first::<i32>(&mut conn)
        .expect("Failed to get level ID");

    let req = test::TestRequest::patch()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({ "background_level": level_id }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 400);
}

#[actix_web::test]
async fn reset_background_level_to_zero() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    let (level_uuid, _) = create_test_aredl_level_with_record(&mut conn, user_id).await;

    let level_id = aredl::levels::table
        .filter(aredl::levels::id.eq(level_uuid))
        .select(aredl::levels::level_id)
        .first::<i32>(&mut conn)
        .unwrap();

    diesel::update(users::table.filter(users::id.eq(user_id)))
        .set(users::background_level.eq(level_id))
        .execute(&mut conn)
        .expect("Failed to set initial background level");

    let req = test::TestRequest::patch()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({ "background_level": 0 }))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let updated: serde_json::Value = test::read_body_json(resp).await;
    assert!(updated["background_level"] == 0);
}
