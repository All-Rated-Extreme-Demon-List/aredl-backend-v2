use crate::schema::users;
#[cfg(test)]
use crate::{test_utils::*, users::test_utils::create_test_user};
#[cfg(test)]
use actix_web::test;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl};

#[actix_web::test]
async fn get_profile() {
    let (app, db, _, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let req = test::TestRequest::get()
        .uri(format!("/arepl/profile/{user}").as_str())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(body["id"], user.to_string(), "IDs do not match!");
}

#[actix_web::test]
async fn get_profile_by_discord_id() {
    let (app, db, _, _) = init_test_app().await;
    let (user, _) = create_test_user(&db, None).await;
    let discord_id = "1234567890";

    diesel::update(users::table.filter(users::id.eq(user)))
        .set(users::discord_id.eq(Some(discord_id)))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set discord id");

    let req = test::TestRequest::get()
        .uri(format!("/arepl/profile/{discord_id}").as_str())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(body["id"], user.to_string(), "IDs do not match!");
}
