use actix_http::StatusCode;
#[cfg(test)]
use {
    crate::{
        auth::{create_test_token, Permission},
        roles::test_utils::{
            add_permission_to_role, create_test_role, create_test_role_inheriting,
        },
        test_utils::{assert_error_response, init_test_app},
        users::test_utils::{create_test_user, TEST_STAFF_ROLE_PRIVILEGE_LEVEL},
    },
    actix_web::test::{self, read_body_json},
};

#[actix_web::test]
async fn list_role_permissions() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let role_id = create_test_role(&db, 10).await;
    add_permission_to_role(&db, role_id, Permission::LevelNotesModify).await;

    let req = test::TestRequest::get()
        .uri(&format!("/roles/{role_id}/permissions"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let permissions: Vec<String> = read_body_json(resp).await;
    assert_eq!(permissions, vec![Permission::LevelNotesModify.to_string()]);
}

#[actix_web::test]
async fn list_resolved_role_permissions_includes_inherited_permissions() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let base_role_id = create_test_role(&db, 5).await;
    add_permission_to_role(&db, base_role_id, Permission::LevelNotesModify).await;

    let role_id = create_test_role_inheriting(&db, 10, base_role_id).await;
    add_permission_to_role(&db, role_id, Permission::LevelUpdatesModify).await;

    let req = test::TestRequest::get()
        .uri(&format!("/roles/{role_id}/permissions/resolved"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let permissions: Vec<String> = read_body_json(resp).await;
    assert_eq!(
        permissions,
        vec![
            Permission::LevelNotesModify.to_string(),
            Permission::LevelUpdatesModify.to_string(),
        ]
    );
}

#[actix_web::test]
async fn add_role_permissions() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let role_id = create_test_role(&db, 10).await;

    let req = test::TestRequest::patch()
        .uri(&format!("/roles/{role_id}/permissions"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(vec![Permission::LevelNotesModify.to_string()])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let permissions: Vec<String> = read_body_json(resp).await;
    assert_eq!(permissions, vec![Permission::LevelNotesModify.to_string()]);
}

#[actix_web::test]
async fn set_role_permissions_replaces_direct_permissions() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let role_id = create_test_role(&db, 10).await;
    add_permission_to_role(&db, role_id, Permission::LevelNotesModify).await;

    let req = test::TestRequest::post()
        .uri(&format!("/roles/{role_id}/permissions"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(vec![Permission::LevelUpdatesModify.to_string()])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let permissions: Vec<String> = read_body_json(resp).await;
    assert_eq!(
        permissions,
        vec![Permission::LevelUpdatesModify.to_string()]
    );
}

#[actix_web::test]
async fn delete_role_permissions() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let role_id = create_test_role(&db, 10).await;
    add_permission_to_role(&db, role_id, Permission::LevelNotesModify).await;
    add_permission_to_role(&db, role_id, Permission::LevelUpdatesModify).await;

    let req = test::TestRequest::delete()
        .uri(&format!("/roles/{role_id}/permissions"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(vec![Permission::LevelNotesModify.to_string()])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let permissions: Vec<String> = read_body_json(resp).await;
    assert_eq!(
        permissions,
        vec![Permission::LevelUpdatesModify.to_string()]
    );
}

#[actix_web::test]
async fn add_role_permissions_rejects_unknown_permission() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let role_id = create_test_role(&db, 10).await;

    let req = test::TestRequest::patch()
        .uri(&format!("/roles/{role_id}/permissions"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(vec!["unknown_permission"])
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_error_response!(
        resp,
        StatusCode::BAD_REQUEST,
        Some("Unknown permission: unknown_permission"),
    );
}

#[actix_web::test]
async fn add_role_permissions_fails_when_target_role_has_same_privilege_as_user() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let role_id = create_test_role(&db, TEST_STAFF_ROLE_PRIVILEGE_LEVEL).await;

    let req = test::TestRequest::patch()
        .uri(&format!("/roles/{role_id}/permissions"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(vec![Permission::LevelNotesModify.to_string()])
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_error_response!(
        resp,
        StatusCode::FORBIDDEN,
        Some("You do not have sufficient permissions to edit this role."),
    );
}
