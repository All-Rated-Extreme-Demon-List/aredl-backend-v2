#[cfg(test)]
use crate::test_utils::*;
#[cfg(test)]
use actix_web::test;
#[actix_web::test]
async fn get_changelog() {
    let (app, _, _) = init_test_app().await;
    let req = test::TestRequest::get()
        .uri("/aredl/changelog")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success(), "status is {}", resp.status());
}
