#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission},
    roles::{test_utils::create_test_role, Role},
    test_utils::init_test_app,
    users::test_utils::create_test_user,
};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use serde_json::json;

#[actix_web::test]
async fn list_roles() {
    let (app, mut conn, _, _) = init_test_app().await;
    let role1 = create_test_role(&mut conn, 10).await;
    let role2 = create_test_role(&mut conn, 20).await;

    let req = test::TestRequest::get().uri("/roles").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let roles: Vec<Role> = test::read_body_json(resp).await;
    let ids: Vec<i32> = roles.iter().map(|r| r.id).collect();
    assert!(ids.contains(&role1));
    assert!(ids.contains(&role2));
}

#[actix_web::test]
async fn create_role() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&mut conn, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let create_data = json!({"privilege_level": 30, "role_desc": "Tester"});
    let req = test::TestRequest::post()
        .uri("/roles")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let created: Role = test::read_body_json(resp).await;
    assert_eq!(created.role_desc, "Tester", "Role description should match");
}

#[actix_web::test]
async fn update_role() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&mut conn, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let role_id = create_test_role(&mut conn, 30).await;

    let update_data = json!({"role_desc": "Updated"});
    let req = test::TestRequest::patch()
        .uri(&format!("/roles/{}", role_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let updated: Role = test::read_body_json(resp).await;
    assert_eq!(
        updated.role_desc, "Updated",
        "Role description should be updated"
    );
}

#[actix_web::test]
async fn delete_role() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&mut conn, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let role_id: i32 = create_test_role(&mut conn, 30).await;

    let req = test::TestRequest::delete()
        .uri(&format!("/roles/{}", role_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get().uri("/roles").to_request();
    let resp = test::call_service(&app, req).await;
    let roles: Vec<Role> = test::read_body_json(resp).await;
    assert!(
        !roles.iter().any(|r| r.id == role_id),
        "Role {} should be deleted",
        role_id
    );
}
