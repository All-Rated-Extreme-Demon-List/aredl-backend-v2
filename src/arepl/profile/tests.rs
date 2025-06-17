#[cfg(test)]
use crate::{test_utils::*, users::test_utils::create_test_user};
#[cfg(test)]
use actix_web::test;

#[actix_web::test]
async fn get_profile() {
    let (app, mut conn, _auth) = init_test_app().await;
    let (user, _) = create_test_user(&mut conn, None).await;
    let req = test::TestRequest::get()
        .uri(format!("/aredl/profile/{user}").as_str())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
    let body: serde_json::Value = test::read_body_json(resp).await;

    assert_eq!(body["id"], user.to_string(), "IDs do not match!");
}
