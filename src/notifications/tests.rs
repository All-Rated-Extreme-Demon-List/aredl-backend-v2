#[cfg(test)]
use {
    crate::{
        auth::{create_test_token, Permission},
        test_utils::init_test_app,
        users::test_utils::create_test_user,
    },
    actix_web::{http::header, test},
    serde_json::json,
};

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
    let (app, _, _, _) = init_test_app().await;
    let req = ws_request("/notifications/websocket").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 403);
}

#[actix_web::test]
async fn websocket_success() {
    let (app, db, auth, _) = init_test_app().await;
    let (user_id, _) = create_test_user(&db, Some(Permission::NotificationsSubscribe)).await;
    let token = create_test_token(user_id, &auth.jwt_encoding_key).unwrap();

    let req = ws_request("/notifications/websocket")
        .insert_header(("Authorization", format!("Bearer {}", token)))
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status().as_u16(), 101);
}

#[actix_web::test]
async fn notification_broadcast() {
    let (_app, _conn, _auth, notify_tx) = init_test_app().await;
    let mut rx = notify_tx.subscribe();

    let note = crate::notifications::WebsocketNotification {
        notification_type: "test".into(),
        data: json!({"hello": 1}),
    };
    notify_tx.send(note.clone()).unwrap();
    let received = rx.recv().await.unwrap();
    assert_eq!(received.notification_type, note.notification_type);
}
