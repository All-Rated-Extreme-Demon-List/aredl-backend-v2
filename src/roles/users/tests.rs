#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission},
    roles::test_utils::{add_user_to_role, create_test_role},
    test_utils::init_test_app,
    users::test_utils::create_test_user,
    users::BaseUser,
};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
#[cfg(test)]
use uuid::Uuid;

#[actix_web::test]
async fn add_role_users() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let role_id = create_test_role(&db, 10).await;
    let (u1, _) = create_test_user(&db, None).await;
    let (u2, _) = create_test_user(&db, None).await;

    let req = test::TestRequest::patch()
        .uri(&format!("/roles/{}/users", role_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&vec![u1, u2])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let users: Vec<BaseUser> = test::read_body_json(resp).await;
    let ids: Vec<Uuid> = users.iter().map(|u| u.id).collect();
    assert!(ids.contains(&u1) && ids.contains(&u2));
}

#[actix_web::test]
async fn set_role_users() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let role_id = create_test_role(&db, 10).await;
    let (u1, _) = create_test_user(&db, None).await;
    add_user_to_role(&db, role_id, u1).await;
    let (u2, _) = create_test_user(&db, None).await;

    let req = test::TestRequest::post()
        .uri(&format!("/roles/{}/users", role_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&vec![u2])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let users: Vec<BaseUser> = test::read_body_json(resp).await;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, u2);
}

#[actix_web::test]
async fn delete_role_users() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::RoleManage)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let role_id = create_test_role(&db, 10).await;
    let (u1, _) = create_test_user(&db, None).await;
    let (u2, _) = create_test_user(&db, None).await;
    add_user_to_role(&db, role_id, u1).await;
    add_user_to_role(&db, role_id, u2).await;

    let req = test::TestRequest::delete()
        .uri(&format!("/roles/{}/users", role_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&vec![u1])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let users: Vec<BaseUser> = test::read_body_json(resp).await;
    assert_eq!(users.len(), 1);
    assert_eq!(users[0].id, u2);
}
