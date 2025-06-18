#[cfg(test)]
use crate::{
    auth::create_test_token,
    clans::test_utils::{create_test_clan, create_test_clan_invite, create_test_clan_member},
    schema::{clan_invites, clan_members, notifications},
    test_utils::init_test_app,
    users::test_utils::create_test_user,
};
#[cfg(test)]
use actix_web::test;
#[cfg(test)]
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};
#[cfg(test)]
#[cfg(test)]
#[actix_web::test]
async fn list_invites() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (owner_id, _) = create_test_user(&mut conn, None).await;
    let (user_id, _) = create_test_user(&mut conn, None).await;

    let clan_id = create_test_clan(&mut conn).await;
    create_test_clan_member(&mut conn, clan_id, owner_id, 2).await;
    let invite_id = create_test_clan_invite(&mut conn, clan_id, user_id, owner_id).await;

    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let req = test::TestRequest::get()
        .uri("/users/@me/clan/invites")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 1);
    assert_eq!(body[0]["id"].as_str().unwrap(), invite_id.to_string());
}

#[actix_web::test]
async fn accept_invite() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (owner_id, _) = create_test_user(&mut conn, None).await;
    let (user_id, _) = create_test_user(&mut conn, None).await;

    let clan_id = create_test_clan(&mut conn).await;
    create_test_clan_member(&mut conn, clan_id, owner_id, 2).await;
    let invite_id = create_test_clan_invite(&mut conn, clan_id, user_id, owner_id).await;

    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let req = test::TestRequest::post()
        .uri(&format!("/users/@me/clan/invites/{invite_id}/accept"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let member_count: i64 = clan_members::table
        .filter(clan_members::clan_id.eq(clan_id))
        .filter(clan_members::user_id.eq(user_id))
        .count()
        .get_result(&mut conn)
        .unwrap();
    assert_eq!(member_count, 1);

    let invite_count: i64 = clan_invites::table
        .filter(clan_invites::user_id.eq(user_id))
        .count()
        .get_result(&mut conn)
        .unwrap();
    assert_eq!(invite_count, 0);

    let notif_count: i64 = notifications::table
        .filter(notifications::user_id.eq(owner_id))
        .count()
        .get_result(&mut conn)
        .unwrap();
    assert_eq!(notif_count, 1);
}

#[actix_web::test]
async fn reject_invite() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (owner_id, _) = create_test_user(&mut conn, None).await;
    let (user_id, _) = create_test_user(&mut conn, None).await;

    let clan_id = create_test_clan(&mut conn).await;
    create_test_clan_member(&mut conn, clan_id, owner_id, 2).await;
    let invite_id = create_test_clan_invite(&mut conn, clan_id, user_id, owner_id).await;

    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let req = test::TestRequest::post()
        .uri(&format!("/users/@me/clan/invites/{invite_id}/reject"))
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let invite_count: i64 = clan_invites::table
        .filter(clan_invites::user_id.eq(user_id))
        .count()
        .get_result(&mut conn)
        .unwrap();
    assert_eq!(invite_count, 0);

    let member_count: i64 = clan_members::table
        .filter(clan_members::clan_id.eq(clan_id))
        .filter(clan_members::user_id.eq(user_id))
        .count()
        .get_result(&mut conn)
        .unwrap();
    assert_eq!(member_count, 0);
}

#[actix_web::test]
async fn leave_clan() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (owner_id, _) = create_test_user(&mut conn, None).await;
    let (user_id, _) = create_test_user(&mut conn, None).await;

    let clan_id = create_test_clan(&mut conn).await;
    create_test_clan_member(&mut conn, clan_id, owner_id, 2).await;
    create_test_clan_member(&mut conn, clan_id, user_id, 1).await;

    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let req = test::TestRequest::post()
        .uri("/users/@me/clan/leave")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let member_count: i64 = clan_members::table
        .filter(clan_members::clan_id.eq(clan_id))
        .filter(clan_members::user_id.eq(user_id))
        .count()
        .get_result(&mut conn)
        .unwrap();
    assert_eq!(member_count, 0);
}

#[actix_web::test]
async fn leave_clan_not_member() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, None).await;

    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let req = test::TestRequest::post()
        .uri("/users/@me/clan/leave")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 404);
}

#[actix_web::test]
async fn leave_clan_owner_forbidden() {
    let (app, mut conn, auth, _) = init_test_app().await;
    let (owner_id, _) = create_test_user(&mut conn, None).await;

    let clan_id = create_test_clan(&mut conn).await;
    create_test_clan_member(&mut conn, clan_id, owner_id, 2).await;

    let token = create_test_token(owner_id, &auth.jwt_encoding_key).unwrap();
    let req = test::TestRequest::post()
        .uri("/users/@me/clan/leave")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 403);
}
