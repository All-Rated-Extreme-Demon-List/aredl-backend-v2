#[cfg(test)]
use crate::{
    roles::test_utils::create_test_role_with_user, test_utils::init_test_app,
    users::names::RoleResolved,
};
#[cfg(test)]
use actix_web::{self, test};

#[actix_web::test]
async fn list_names() {
    let (app, mut conn, _, _) = init_test_app().await;

    let (role_id_1, user_id_1) = create_test_role_with_user(&mut conn, 0).await;
    let (role_id_2, user_id_2) = create_test_role_with_user(&mut conn, 0).await;

    let req = test::TestRequest::get().uri("/users/names").to_request();

    let res = test::call_service(&app, req).await;
    assert!(res.status().is_success(), "status is {}", res.status());

    let names: Vec<RoleResolved> = actix_web::test::read_body_json(res).await;

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
