use actix_http::StatusCode;
#[cfg(test)]
use {
    crate::{
        auth::{create_test_token, Permission},
        clans::test_utils::{
            count_test_clan_members, create_test_clan, create_test_clan_member,
            set_test_clan_member_timestamps, test_clan_member, test_clan_member_user_ids,
        },
        test_utils::{assert_error_response, init_test_app},
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
    chrono::{Duration, Timelike as _, Utc},
    serde_json::json,
    uuid::Uuid,
};

#[actix_web::test]
async fn add_members() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::ClanModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let clan_id = create_test_clan(&db).await;
    let (user_id, _) = create_test_user(&db, None).await;
    let req = test::TestRequest::post()
        .uri(&format!("/clans/{clan_id}/members"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(vec![user_id])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let members: Vec<Uuid> = read_body_json(resp).await;
    assert!(members.contains(&user_id));

    let count = count_test_clan_members(&db, clan_id, user_id);
    assert_eq!(count, 1);
}

#[actix_web::test]
async fn list_members() {
    let (app, db, _auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (user_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, user_id, 0).await;

    let req = test::TestRequest::get()
        .uri(&format!("/clans/{clan_id}/members"))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let members: serde_json::Value = read_body_json(resp).await;
    assert!(members
        .as_array()
        .unwrap()
        .iter()
        .any(|m| m["id"].as_str().unwrap() == user_id.to_string()));
}

#[actix_web::test]
async fn set_members() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::ClanModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let clan_id = create_test_clan(&db).await;
    let (u1, _) = create_test_user(&db, None).await;
    let (u2, _) = create_test_user(&db, None).await;

    let req = test::TestRequest::patch()
        .uri(&format!("/clans/{clan_id}/members"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(vec![u1, u2])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let members: Vec<Uuid> = read_body_json(resp).await;
    assert_eq!(members.len(), 2);
    assert!(members.contains(&u1));
    assert!(members.contains(&u2));
}

#[actix_web::test]
async fn set_members_removes_missing_members() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::ClanModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let clan_id = create_test_clan(&db).await;
    let (u1, _) = create_test_user(&db, None).await;
    let (u2, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, u1, 0).await;
    create_test_clan_member(&db, clan_id, u2, 0).await;

    let req = test::TestRequest::patch()
        .uri(&format!("/clans/{clan_id}/members"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(vec![u2])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let members: Vec<Uuid> = read_body_json(resp).await;
    assert!(!members.contains(&u1));
    assert!(members.contains(&u2));

    let remaining = test_clan_member_user_ids(&db, clan_id);
    assert_eq!(remaining.len(), 1);
    assert!(remaining.contains(&u2));
}

#[actix_web::test]
async fn set_members_preserves_metadata() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::ClanModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let clan_id = create_test_clan(&db).await;

    let (existing_user, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, existing_user, 1).await;
    let preserved_timestamp = (Utc::now() - Duration::days(7)).with_nanosecond(0).unwrap();
    set_test_clan_member_timestamps(&db, clan_id, existing_user, preserved_timestamp);

    let (new_user, _) = create_test_user(&db, None).await;

    let req = test::TestRequest::patch()
        .uri(&format!("/clans/{clan_id}/members"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(vec![existing_user, new_user])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let member = test_clan_member(&db, clan_id, existing_user);

    assert_eq!(member.role, 1);
    assert_eq!(member.created_at, preserved_timestamp);
    assert_eq!(member.updated_at, preserved_timestamp);
}

#[actix_web::test]
async fn delete_members() {
    let (app, db, auth, _) = init_test_app().await;
    let (owner_id, _) = create_test_user(&db, None).await;
    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();
    let clan_id = create_test_clan(&db).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let (member_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, member_id, 0).await;

    let req = test::TestRequest::delete()
        .uri(&format!("/clans/{clan_id}/members"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(vec![member_id])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let remaining: Vec<Uuid> = read_body_json(resp).await;
    assert!(!remaining.contains(&member_id));
}

#[actix_web::test]
async fn delete_members_unauthorized() {
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let (member_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, member_id, 0).await;
    let token = create_test_token(member_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::delete()
        .uri(&format!("/clans/{clan_id}/members"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(vec![owner_id])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(resp, StatusCode::FORBIDDEN, None).await;
}

#[actix_web::test]
async fn invite_member() {
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();
    let (user_id, _) = create_test_user(&db, None).await;

    let req = test::TestRequest::post()
        .uri(&format!("/clans/{clan_id}/members/invite"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"user_id": user_id}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let invite: serde_json::Value = read_body_json(resp).await;
    assert_eq!(invite["user_id"].as_str().unwrap(), user_id.to_string());
}

#[actix_web::test]
async fn invite_member_unauthorized() {
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let (member_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, member_id, 0).await;
    let token = create_test_token(member_id, &auth.jwt_encoding_key).unwrap();
    let (user_id, _) = create_test_user(&db, None).await;

    let req = test::TestRequest::post()
        .uri(&format!("/clans/{clan_id}/members/invite"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"user_id": user_id}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(resp, StatusCode::FORBIDDEN, None).await;
}

#[actix_web::test]
async fn edit_member() {
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();
    let (member_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, member_id, 0).await;

    let req = test::TestRequest::patch()
        .uri(&format!("/clans/{clan_id}/members/{member_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"role": 1}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let member: serde_json::Value = read_body_json(resp).await;
    assert_eq!(member["role"], 1);
}

#[actix_web::test]
async fn edit_member_unauthorized() {
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let (member_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, member_id, 0).await;
    let token = create_test_token(member_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::patch()
        .uri(&format!("/clans/{clan_id}/members/{owner_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"role": 1}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(resp, StatusCode::FORBIDDEN, None).await;
}

#[actix_web::test]
async fn edit_member_transfer_ownership() {
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    let (member_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    create_test_clan_member(&db, clan_id, member_id, 1).await;
    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::patch()
        .uri(&format!("/clans/{clan_id}/members/{member_id}"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"role": 2}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let member: serde_json::Value = read_body_json(resp).await;
    assert_eq!(member["role"], 2);

    let old_owner_role = test_clan_member(&db, clan_id, owner_id).role;
    assert_eq!(old_owner_role, 1);
}

#[actix_web::test]
async fn invite_member_already_in_clan() {
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    let (user_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    create_test_clan_member(&db, clan_id, user_id, 0).await;
    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::post()
        .uri(&format!("/clans/{clan_id}/members/invite"))
        .insert_header(("Authorization", format!("Bearer {token}")))
        .set_json(json!({"user_id": user_id}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_error_response(
        resp,
        StatusCode::CONFLICT,
        Some("This user is already in a clan"),
    )
    .await;
}
