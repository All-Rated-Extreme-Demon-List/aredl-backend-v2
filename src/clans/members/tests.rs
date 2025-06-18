#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission},
    clans::test_utils::create_test_clan,
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
    let (app, mut conn, auth, _) = init_test_app().await;
    let (staff_id, _) = create_test_user(&mut conn, Some(Permission::ClanModify)).await;
    let token = create_test_token(staff_id, &auth.jwt_encoding_key).unwrap();
    let clan_id = create_test_clan(&mut conn).await;
    let (user_id, _) = create_test_user(&mut conn, None).await;

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
        .get_result(&mut conn)
        .unwrap();
    assert_eq!(count, 1);
}
