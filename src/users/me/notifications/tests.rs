use super::test_utils::{count_test_notifications, create_test_notification};
#[cfg(test)]
use {
    crate::{
        auth::create_test_token, test_utils::init_test_app,
        users::me::notifications::NotificationType, users::test_utils::create_test_user,
    },
    actix_web::test::{self, read_body_json},
};

#[actix_web::test]
async fn list_notifications() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, None).await;

    create_test_notification(&db, user_id, "One", NotificationType::Info);
    create_test_notification(&db, user_id, "Two", NotificationType::Success);

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

    create_test_notification(&db, user_id, "One", NotificationType::Info);
    create_test_notification(&db, user_id, "Two", NotificationType::Failure);

    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();
    let req = test::TestRequest::post()
        .uri("/users/@me/notifications/clear")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let remaining = count_test_notifications(&db, user_id);
    assert_eq!(remaining, 0);
}
