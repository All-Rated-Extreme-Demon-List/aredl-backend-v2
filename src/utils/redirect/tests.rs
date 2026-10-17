#[cfg(test)]
use {
    super::redirect_prefix,
    actix_web::{
        http::{header, StatusCode},
        test, web, App,
    },
};

#[actix_web::test]
async fn redirect_profile() {
    let app = test::init_service(
        App::new()
            .service(web::scope("/api/{list}").service(redirect_prefix("/clan", "/profile/clan"))),
    )
    .await;

    let req = test::TestRequest::get()
        .uri("/api/aredl/clan/a%20b/levels/a%2Fb/records?filter=a%26b&filter=c%2Fd&empty=")
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), StatusCode::PERMANENT_REDIRECT);
    assert_eq!(
        resp.headers().get(header::LOCATION).unwrap(),
        "/api/aredl/profile/clan/a%20b/levels/a%2Fb/records?filter=a%26b&filter=c%2Fd&empty="
    );
}

#[actix_web::test]
async fn redirect_allowed_methods() {
    let app =
        test::init_service(App::new().service(redirect_prefix("/country", "/profile/country")))
            .await;

    let resp = test::call_service(
        &app,
        test::TestRequest::default()
            .method(actix_web::http::Method::HEAD)
            .uri("/country/840")
            .to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::PERMANENT_REDIRECT);
    assert_eq!(
        resp.headers().get(header::LOCATION).unwrap(),
        "/profile/country/840"
    );
    assert!(test::read_body(resp).await.is_empty());

    let resp = test::call_service(
        &app,
        test::TestRequest::post().uri("/country/840").to_request(),
    )
    .await;
    assert_eq!(resp.status(), StatusCode::METHOD_NOT_ALLOWED);
    assert!(resp.headers().get(header::LOCATION).is_none());
    assert!(test::read_body(resp).await.is_empty());
}
