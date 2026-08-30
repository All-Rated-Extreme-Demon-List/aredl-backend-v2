#[cfg(test)]
use {
    crate::{
        auth::{create_test_token, permission, Permission},
        roles::{
            test_utils::{add_user_to_role, create_test_role, create_test_role_with_permission},
            Role, RoleResolved,
        },
        test_utils::{assert_error_response, init_test_app},
        users::test_utils::{
            create_test_full_reviewer, create_test_hidden_reviewer, create_test_user,
            TEST_STAFF_ROLE_PRIVILEGE_LEVEL,
        },
    },
    actix_http::StatusCode,
    actix_web::test::{self, read_body_json},
    serde_json::json,
};

#[actix_web::test]
async fn list_roles() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleAssign)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let role1 = create_test_role_with_permission(&db, 10, Permission::LevelNotesModify).await;
    let role2 = create_test_role(&db, 20).await;

    let req = test::TestRequest::get()
        .uri("/roles")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let roles: Vec<RoleResolved> = read_body_json(resp).await;
    let ids: Vec<i32> = roles.iter().map(|r| r.role.id).collect();
    assert!(ids.contains(&role1));
    assert!(ids.contains(&role2));

    let role1 = roles
        .iter()
        .find(|role| role.role.id == role1)
        .expect("role should be returned");

    assert_eq!(
        role1.permissions,
        vec![Permission::LevelNotesModify.to_string()]
    );
}

#[actix_web::test]
async fn create_role() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let create_data = json!({"privilege_level": 30, "role_desc": "Tester", "hide": false});
    let req = test::TestRequest::post()
        .uri("/roles")
        .insert_header(("Authorization", format!("Bearer {token}")))
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
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let role_id = create_test_role(&db, 30).await;

    let update_data = json!({"role_desc": "Updated"});
    let req = test::TestRequest::patch()
        .uri(&format!("/roles/{role_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
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
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let role_id: i32 = create_test_role(&db, 30).await;

    let req = test::TestRequest::delete()
        .uri(&format!("/roles/{role_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req = test::TestRequest::get()
        .uri("/roles")
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();
    let resp = test::call_service(&app, req).await;
    let roles: Vec<RoleResolved> = read_body_json(resp).await;
    assert!(
        !roles.iter().any(|r| r.role.id == role_id),
        "Role {role_id} should be deleted"
    );
}

#[actix_web::test]
async fn create_role_rejects_same_or_higher_privilege_than_user() {
    let (app, db, auth, _) = init_test_app().await;

    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    for (privilege_level, role_desc) in [
        (TEST_STAFF_ROLE_PRIVILEGE_LEVEL, "Same Level Role"),
        (TEST_STAFF_ROLE_PRIVILEGE_LEVEL + 1, "Higher Level Role"),
    ] {
        let create_data =
            json!({"privilege_level": privilege_level, "role_desc": role_desc, "hide": false});

        let req = test::TestRequest::post()
            .uri("/roles")
            .insert_header(("Authorization", format!("Bearer {token}")))
            .set_json(&create_data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_error_response!(
            resp,
            StatusCode::FORBIDDEN,
            Some("You can not create a role with higher permissions than yourself."),
        );
    }
}

#[actix_web::test]
async fn update_role_fails_when_target_role_has_same_privilege_as_user() {
    let (app, db, auth, _) = init_test_app().await;

    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let lvl = TEST_STAFF_ROLE_PRIVILEGE_LEVEL;
    let role_id = create_test_role(&db, lvl).await;

    let req = test::TestRequest::patch()
        .uri(&format!("/roles/{role_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"role_desc": "Should Not Work"}))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_error_response!(
        resp,
        StatusCode::FORBIDDEN,
        Some("You do not have sufficient permissions to edit this role."),
    );
}

#[actix_web::test]
async fn delete_role_fails_when_target_role_has_same_privilege_as_user() {
    let (app, db, auth, _) = init_test_app().await;

    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();

    let lvl = TEST_STAFF_ROLE_PRIVILEGE_LEVEL;
    let role_id = create_test_role(&db, lvl).await;

    let req = test::TestRequest::delete()
        .uri(&format!("/roles/{role_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_error_response!(
        resp,
        StatusCode::FORBIDDEN,
        Some("You do not have sufficient permissions to edit this role."),
    );
}

#[actix_web::test]
async fn find_reviewer_visibility_excludes_visible_reviewers_and_mixed_role_users() {
    let (_app, db, _auth, _) = init_test_app().await;

    let (hidden_only_user, _) = create_test_hidden_reviewer(&db).await;
    let (visible_user, _) = create_test_full_reviewer(&db).await;
    let (mixed_user, _) = create_test_user(&db, None).await;

    let review_role = create_test_role_with_permission(&db, 0, Permission::SubmissionReview).await;
    let visibility_role =
        create_test_role_with_permission(&db, 0, Permission::SubmissionReviewerVisible).await;
    add_user_to_role(&db, review_role, mixed_user).await;
    add_user_to_role(&db, visibility_role, mixed_user).await;

    let conn = &mut db.connection().unwrap();
    let reviewers = permission::get_users_with_permission(conn, Permission::SubmissionReview)
        .expect("Failed to load reviewers");
    let visible_permissions =
        permission::get_users_with_permission(conn, Permission::SubmissionReviewerVisible)
            .expect("Failed to load visible reviewers");
    let visible_reviewers = reviewers
        .intersection(&visible_permissions)
        .copied()
        .collect::<std::collections::HashSet<_>>();
    let hidden_reviewers = reviewers
        .difference(&visible_reviewers)
        .copied()
        .collect::<std::collections::HashSet<_>>();

    assert!(hidden_reviewers.contains(&hidden_only_user));
    assert!(visible_reviewers.contains(&visible_user));
    assert!(visible_reviewers.contains(&mixed_user));
    assert!(!hidden_reviewers.contains(&visible_user));
    assert!(!hidden_reviewers.contains(&mixed_user));
}
