#[cfg(test)]
use {crate::test_utils::init_test_app, actix_web::test};

#[actix_web::test]
async fn health_ok() {
    let (app, _, _, _) = init_test_app().await;
    let req = test::TestRequest::get().uri("/health").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}
