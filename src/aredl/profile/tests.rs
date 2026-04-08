#[cfg(test)]
use {
    crate::{
        aredl::levels::test_utils::create_test_level_with_record,
        auth::{create_test_token, Permission},
        roles::test_utils::{add_user_to_role, create_test_hidden_role, create_test_role},
        schema::users,
        test_utils::*,
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
};

#[actix_web::test]
async fn get_profile() {
    let (app, db, _, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let req = test::TestRequest::get()
        .uri(format!("/aredl/profile/{user}").as_str())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    assert_eq!(body["id"], user.to_string(), "IDs do not match!");
}

#[actix_web::test]
async fn get_profile_by_discord_id() {
    let (app, db, _, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let discord_id = "1234567890";

    diesel::update(users::table.filter(users::id.eq(user)))
        .set(users::discord_id.eq(Some(discord_id)))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set discord id");

    let req = test::TestRequest::get()
        .uri(format!("/aredl/profile/{discord_id}").as_str())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = read_body_json(resp).await;

    assert_eq!(body["id"], user.to_string(), "IDs do not match!");
}

#[actix_web::test]
async fn get_profile_hides_hidden_roles_except_for_role_manage() {
    let (app, db, auth, _) = init_test_app().await;
    let (target_user, _) = create_test_user(&db, None).await;
    let (normal_requester, _) = create_test_user(&db, None).await;
    let (manager_requester, _) = create_test_user(&db, Some(Permission::RoleManage)).await;

    let hidden_role_id = create_test_hidden_role(&db, 5).await;
    let visible_role_id = create_test_role(&db, 4).await;

    add_user_to_role(&db, hidden_role_id, target_user).await;
    add_user_to_role(&db, visible_role_id, target_user).await;

    let normal_token = create_test_token(normal_requester, &auth.jwt_encoding_key).unwrap();
    let manager_token = create_test_token(manager_requester, &auth.jwt_encoding_key).unwrap();

    let assert_hidden_role_is_not_exposed = |roles: &Vec<serde_json::Value>| {
        assert!(
            !roles.iter().any(|r| r["id"] == hidden_role_id),
            "Hidden role should not be present in profile response"
        );

        let visible_role = roles
            .iter()
            .find(|r| r["id"] == visible_role_id)
            .expect("Visible role should be present in profile response");

        assert_eq!(visible_role["hide"], false);
    };

    let assert_hidden_role_is_exposed = |roles: &Vec<serde_json::Value>| {
        let hidden_role = roles
            .iter()
            .find(|r| r["id"] == hidden_role_id)
            .expect("Hidden role should be present for role_manage users");
        let visible_role = roles
            .iter()
            .find(|r| r["id"] == visible_role_id)
            .expect("Visible role should be present in profile response");

        assert_eq!(hidden_role["hide"], true);
        assert_eq!(visible_role["hide"], false);
    };

    let anon_req = test::TestRequest::get()
        .uri(format!("/aredl/profile/{target_user}").as_str())
        .to_request();
    let anon_resp = test::call_service(&app, anon_req).await;
    assert!(anon_resp.status().is_success());
    let anon_body: serde_json::Value = read_body_json(anon_resp).await;
    let anon_roles = anon_body["roles"].as_array().unwrap();
    assert_hidden_role_is_not_exposed(anon_roles);

    let normal_req = test::TestRequest::get()
        .uri(format!("/aredl/profile/{target_user}").as_str())
        .insert_header(("Authorization", format!("Bearer {}", normal_token)))
        .to_request();
    let normal_resp = test::call_service(&app, normal_req).await;
    assert!(normal_resp.status().is_success());
    let normal_body: serde_json::Value = read_body_json(normal_resp).await;
    let normal_roles = normal_body["roles"].as_array().unwrap();
    assert_hidden_role_is_not_exposed(normal_roles);

    let manager_req = test::TestRequest::get()
        .uri(format!("/aredl/profile/{target_user}").as_str())
        .insert_header(("Authorization", format!("Bearer {}", manager_token)))
        .to_request();
    let manager_resp = test::call_service(&app, manager_req).await;
    assert!(manager_resp.status().is_success());
    let manager_body: serde_json::Value = read_body_json(manager_resp).await;
    let manager_roles = manager_body["roles"].as_array().unwrap();
    assert_hidden_role_is_exposed(manager_roles);
}

#[actix_web::test]
async fn get_profile_includes_badges_and_featured_badge() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;
    let token =
        create_test_token(user_id, &auth.jwt_encoding_key).expect("Failed to generate token");

    create_test_level_with_record(&db, user_id).await;

    let sync_req = test::TestRequest::post()
        .uri("/users/@me/sync")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let sync_resp = test::call_service(&app, sync_req).await;
    assert!(sync_resp.status().is_success());

    let badge_code = "global.level_completion.1";
    let feature_req = test::TestRequest::patch()
        .uri("/users/@me")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&serde_json::json!({ "featured_badge_code": badge_code }))
        .to_request();
    let feature_resp = test::call_service(&app, feature_req).await;
    assert!(feature_resp.status().is_success());

    let profile_req = test::TestRequest::get()
        .uri(format!("/aredl/profile/{user_id}").as_str())
        .to_request();
    let profile_resp = test::call_service(&app, profile_req).await;
    assert!(profile_resp.status().is_success());

    let body: serde_json::Value = read_body_json(profile_resp).await;
    assert_eq!(body["featured_badge_code"], badge_code);
    assert!(body["badges"]
        .as_array()
        .unwrap()
        .iter()
        .any(|badge| badge["badge_code"] == badge_code));
}
