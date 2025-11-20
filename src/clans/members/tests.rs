#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission},
    clans::test_utils::{create_test_clan, create_test_clan_member},
    schema::clan_members,
    test_utils::init_test_app,
    users::test_utils::create_test_user,
};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
use uuid::Uuid;

#[actix_web::test]
async fn add_members() {
    let (app, db, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&db, Some(Permission::ClanModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let clan_id = create_test_clan(&db).await;
    let (user_id, _) = create_test_user(&db, None).await;
    let req = test::TestRequest::post()
        .uri(&format!("/clans/{}/members", clan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&vec![user_id])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let members: Vec<Uuid> = test::read_body_json(resp).await;
    assert!(members.contains(&user_id));

    let count: i64 = clan_members::table
        .filter(clan_members::clan_id.eq(clan_id))
        .filter(clan_members::user_id.eq(user_id))
        .count()
        .get_result(&mut db.connection().unwrap())
        .unwrap();
    assert_eq!(count, 1);
}

#[actix_web::test]
async fn list_members() {
    let (app, db, _auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (user_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, user_id, 0).await;

    let req = test::TestRequest::get()
        .uri(&format!("/clans/{}/members", clan_id))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let members: serde_json::Value = test::read_body_json(resp).await;
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
        .uri(&format!("/clans/{}/members", clan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&vec![u1, u2])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let members: Vec<Uuid> = test::read_body_json(resp).await;
    assert_eq!(members.len(), 2);
    assert!(members.contains(&u1));
    assert!(members.contains(&u2));
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
        .uri(&format!("/clans/{}/members", clan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&vec![member_id])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let remaining: Vec<Uuid> = test::read_body_json(resp).await;
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
        .uri(&format!("/clans/{}/members", clan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&vec![owner_id])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 403);
}

#[actix_web::test]
async fn invite_member() {
    use serde_json::json;
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();
    let (user_id, _) = create_test_user(&db, None).await;

    let req = test::TestRequest::post()
        .uri(&format!("/clans/{}/members/invite", clan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"user_id": user_id}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let invite: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(invite["user_id"].as_str().unwrap(), user_id.to_string());
}

#[actix_web::test]
async fn invite_member_unauthorized() {
    use serde_json::json;
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let (member_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, member_id, 0).await;
    let token = create_test_token(member_id, &auth.jwt_encoding_key).unwrap();
    let (user_id, _) = create_test_user(&db, None).await;

    let req = test::TestRequest::post()
        .uri(&format!("/clans/{}/members/invite", clan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"user_id": user_id}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 403);
}

#[actix_web::test]
async fn edit_member() {
    use serde_json::json;
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();
    let (member_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, member_id, 0).await;

    let req = test::TestRequest::patch()
        .uri(&format!("/clans/{}/members/{}", clan_id, member_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"role": 1}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let member: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(member["role"], 1);
}

#[actix_web::test]
async fn edit_member_unauthorized() {
    use serde_json::json;
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    let (member_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, member_id, 0).await;
    let token = create_test_token(member_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::patch()
        .uri(&format!("/clans/{}/members/{}", clan_id, owner_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"role": 1}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 403);
}

#[actix_web::test]
async fn edit_member_transfer_ownership() {
    use serde_json::json;
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    let (member_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    create_test_clan_member(&db, clan_id, member_id, 1).await;
    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::patch()
        .uri(&format!("/clans/{}/members/{}", clan_id, member_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"role": 2}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
    let member: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(member["role"], 2);

    let old_owner_role: i32 = clan_members::table
        .filter(clan_members::clan_id.eq(clan_id))
        .filter(clan_members::user_id.eq(owner_id))
        .select(clan_members::role)
        .first(&mut db.connection().unwrap())
        .unwrap();
    assert_eq!(old_owner_role, 1);
}

#[actix_web::test]
async fn invite_member_already_in_clan() {
    use serde_json::json;
    let (app, db, auth, _) = init_test_app().await;
    let clan_id = create_test_clan(&db).await;
    let (owner_id, _) = create_test_user(&db, None).await;
    let (user_id, _) = create_test_user(&db, None).await;
    create_test_clan_member(&db, clan_id, owner_id, 2).await;
    create_test_clan_member(&db, clan_id, user_id, 0).await;
    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();

    let req = test::TestRequest::post()
        .uri(&format!("/clans/{}/members/invite", clan_id))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .set_json(&json!({"user_id": user_id}))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_client_error());
}
