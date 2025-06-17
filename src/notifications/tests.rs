#[cfg(test)]
use crate::{
    auth::{create_test_token, Permission},
    test_utils::init_test_app,
    users::test_utils::create_test_user,
};
#[cfg(test)]
use actix_web::{http::header, test};

#[cfg(test)]
fn ws_request(path: &str) -> test::TestRequest {
    test::TestRequest::get()
        .uri(path)
        .insert_header((header::UPGRADE, "websocket"))
        .insert_header((header::CONNECTION, "upgrade"))
        .insert_header((header::SEC_WEBSOCKET_VERSION, "13"))
        .insert_header((header::SEC_WEBSOCKET_KEY, "testkey=="))
}

#[actix_web::test]
async fn websocket_requires_auth() {
    let (app, _, _) = init_test_app().await;
    let req = ws_request("/notifications/websocket").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 403);
}

#[actix_web::test]
async fn websocket_success() {
    let (app, mut conn, auth) = init_test_app().await;
    let (user_id, _) = create_test_user(&mut conn, Some(Permission::NotificationsSubscribe)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let req = ws_request("/notifications/websocket")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 101);
}
