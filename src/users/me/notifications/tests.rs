#[cfg(test)]
use {
    crate::{
        auth::create_test_token,
        schema::notifications,
        test_utils::init_test_app,
        users::me::notifications::{Notification, NotificationType},
        users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
    diesel::{ExpressionMethods, QueryDsl, RunQueryDsl},
};
#[actix_web::test]
async fn list_notifications() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;

    Notification::create(
        &mut db.connection().unwrap(),
        user_id,
        "One".into(),
        NotificationType::Info,
    )
    .unwrap();
    Notification::create(
        &mut db.connection().unwrap(),
        user_id,
        "Two".into(),
        NotificationType::Success,
    )
    .unwrap();

    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let req = test::TestRequest::get()
        .uri("/users/@me/notifications")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = read_body_json(resp).await;
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[actix_web::test]
async fn clear_notifications() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;

    Notification::create(
        &mut db.connection().unwrap(),
        user_id,
        "One".into(),
        NotificationType::Info,
    )
    .unwrap();
    Notification::create(
        &mut db.connection().unwrap(),
        user_id,
        "Two".into(),
        NotificationType::Failure,
    )
    .unwrap();

    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let req = test::TestRequest::post()
        .uri("/users/@me/notifications/clear")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let remaining: i64 = notifications::table
        .filter(notifications::user_id.eq(user_id))
        .count()
        .get_result(&mut db.connection().unwrap())
        .unwrap();
    assert_eq!(remaining, 0);
}
