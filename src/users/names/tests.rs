#[cfg(test)]
use {
    crate::{
        roles::test_utils::create_test_role_with_user, roles::RoleResolved, schema::roles,
        test_utils::init_test_app,
    },
    actix_web::{self, test::{self, read_body_json}},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
};

#[actix_web::test]
async fn list_names() {
    let (app, db, _, _) = init_test_app().await;

    let (role_id_1, user_id_1) = create_test_role_with_user(&db, 0).await;
    let (role_id_2, user_id_2) = create_test_role_with_user(&db, 0).await;

    let req = test::TestRequest::get().uri("/users/names").to_request();

    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());

    let names: Vec<RoleResolved> = read_body_json(res).await;

    assert_eq!(names.len(), 2, "Expected 2 entries, found {}", names.len());
    let returned_role_ids: Vec<i32> = names.iter().map(|r| r.role.id).collect();
    assert_eq!(
        returned_role_ids,
        vec![role_id_1, role_id_2],
        "Expected role IDs [{}, {}], got {:?}",
        role_id_1,
        role_id_2,
        returned_role_ids
    );

    for (idx, expected_user_id) in [user_id_1, user_id_2].iter().enumerate() {
        let resolved = &names[idx];
        assert_eq!(
            resolved.users.len(),
            1,
            "Role {} should have exactly one user, found {}",
            resolved.role.id,
            resolved.users.len()
        );
        assert_eq!(
            &resolved.users[0].id, expected_user_id,
            "Role {}: expected user ID {}, got {}",
            resolved.role.id, expected_user_id, resolved.users[0].id
        );
    }
}

#[actix_web::test]
async fn list_names_excludes_hidden_roles() {
    let (app, db, _, _) = init_test_app().await;

    let (visible_role_id, visible_user_id) = create_test_role_with_user(&db, 0).await;
    let (hidden_role_id, _hidden_user_id) = create_test_role_with_user(&db, 0).await;

    diesel::update(roles::table.filter(roles::id.eq(hidden_role_id)))
        .set(roles::hide.eq(true))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to hide role");

    let req = test::TestRequest::get().uri("/users/names").to_request();
    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());

    let names: Vec<RoleResolved> = read_body_json(res).await;

    assert_eq!(names.len(), 1, "Expected 1 entry, found {}", names.len());
    assert_eq!(names[0].role.id, visible_role_id);
    assert_eq!(names[0].users.len(), 1);
    assert_eq!(names[0].users[0].id, visible_user_id);
}
