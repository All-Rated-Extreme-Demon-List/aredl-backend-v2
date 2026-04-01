#[cfg(test)]
use {
    crate::{
        auth::{create_test_token, Permission},
        roles::{
            test_utils::{add_user_to_role, create_test_role},
            Role, RoleResolved,
        },
        test_utils::{assert_error_response, init_test_app},
        users::test_utils::{create_test_user, get_permission_privilege_level},
    },
    actix_web::test::{self, read_body_json},
    serde_json::json,
};

#[actix_web::test]
async fn list_roles() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let role1 = create_test_role(&db, 10).await;
    let role2 = create_test_role(&db, 20).await;

    let req = test::TestRequest::get()
        .uri("/roles")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let roles: Vec<RoleResolved> = read_body_json(resp).await;
    let ids: Vec<i32> = roles.iter().map(|r| r.role.id).collect();
    assert!(ids.contains(&role1));
    assert!(ids.contains(&role2));
}

#[actix_web::test]
async fn create_role() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let create_data = json!({"privilege_level": 30, "role_desc": "Tester"});
    let req = test::TestRequest::post()
        .uri("/roles")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let created: Role = read_body_json(resp).await;
    assert_eq!(created.role_desc, "Tester", "Role description should match");
}

#[actix_web::test]
async fn update_role() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let role_id = create_test_role(&db, 30).await;

    let update_data = json!({"role_desc": "Updated"});
    let req = test::TestRequest::patch()
        .uri(&format!("/roles/{}", role_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&update_data)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let updated: Role = read_body_json(resp).await;
    assert_eq!(
        updated.role_desc, "Updated",
        "Role description should be updated"
    );
}

#[actix_web::test]
async fn delete_role() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let role_id: i32 = create_test_role(&db, 30).await;

    let req = test::TestRequest::delete()
        .uri(&format!("/roles/{}", role_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get()
        .uri("/roles")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let roles: Vec<RoleResolved> = read_body_json(resp).await;
    assert!(
        !roles.iter().any(|r| r.role.id == role_id),
        "Role {} should be deleted",
        role_id
    );
}

#[actix_web::test]
async fn create_role_fails_when_new_role_has_same_privilege_as_user() {
    let (app, db, auth, _) = init_test_app().await;

    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let lvl = get_permission_privilege_level(&db, Permission::RoleManage);
    let create_data = json!({"privilege_level": lvl, "role_desc": "Same Level Role"});

    let req = test::TestRequest::post()
        .uri("/roles")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        403,
        Some("You can not create a role with higher permissions than yourself."),
    )
    .await;
}

#[actix_web::test]
async fn create_role_fails_when_new_role_has_higher_privilege_than_user() {
    let (app, db, auth, _) = init_test_app().await;

    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let lvl = get_permission_privilege_level(&db, Permission::RoleManage);
    let create_data = json!({"privilege_level": lvl + 1, "role_desc": "Higher Level Role"});

    let req = test::TestRequest::post()
        .uri("/roles")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&create_data)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        403,
        Some("You can not create a role with higher permissions than yourself."),
    )
    .await;
}

#[actix_web::test]
async fn update_role_fails_when_target_role_has_same_privilege_as_user() {
    let (app, db, auth, _) = init_test_app().await;

    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let lvl = get_permission_privilege_level(&db, Permission::RoleManage);
    let role_id = create_test_role(&db, lvl).await;

    let req = test::TestRequest::patch()
        .uri(&format!("/roles/{}", role_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"role_desc": "Should Not Work"}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        403,
        Some("You do not have sufficient permissions to edit this role."),
    )
    .await;
}

#[actix_web::test]
async fn delete_role_fails_when_target_role_has_same_privilege_as_user() {
    let (app, db, auth, _) = init_test_app().await;

    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let lvl = get_permission_privilege_level(&db, Permission::RoleManage);
    let role_id = create_test_role(&db, lvl).await;

    let req = test::TestRequest::delete()
        .uri(&format!("/roles/{}", role_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        403,
        Some("You do not have sufficient permissions to edit this role."),
    )
    .await;
}

#[actix_web::test]
async fn find_all_base_reviewers_excludes_full_reviewers_and_mixed_role_users() {
    let (_app, db, _auth, _) = init_test_app().await;

    let (base_only_user, _) = create_test_user(&db, Some(Permission::SubmissionReviewBase)).await;
    let (full_user, _) = create_test_user(&db, Some(Permission::SubmissionReviewFull)).await;
    let (mixed_user, _) = create_test_user(&db, None).await;

    let base_level = get_permission_privilege_level(&db, Permission::SubmissionReviewBase);
    let full_level = get_permission_privilege_level(&db, Permission::SubmissionReviewFull);

    let base_role = create_test_role(&db, base_level).await;
    let full_role = create_test_role(&db, full_level).await;
    add_user_to_role(&db, base_role, mixed_user).await;
    add_user_to_role(&db, full_role, mixed_user).await;

    let reviewer_sets =
        RoleResolved::find_all_base_reviewers(&mut db.connection().unwrap()).unwrap();

    assert!(reviewer_sets.base_reviewers.contains(&base_only_user));
    assert!(reviewer_sets.full_reviewers.contains(&full_user));
    assert!(reviewer_sets.full_reviewers.contains(&mixed_user));
    assert!(!reviewer_sets.base_reviewers.contains(&full_user));
    assert!(!reviewer_sets.base_reviewers.contains(&mixed_user));
}
