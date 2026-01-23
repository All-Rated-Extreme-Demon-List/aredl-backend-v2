#[cfg(test)]
use {
    crate::{
        error_handler::ApiError,
        page_helper::{PageQuery, Paginated},
        test_utils::assert_error_response,
    },
    actix_web::{http::header, test, web, App, HttpResponse},
    super::*,
};

#[test]
async fn page_query_defaults_and_offset() {
    let q: PageQuery<20> = PageQuery {
        per_page: None,
        page: None,
    };
    assert_eq!(q.per_page(), 20);
    assert_eq!(q.page(), 1);
    assert_eq!(q.offset(), 0);

    let q2 = PageQuery::<20> {
        per_page: Some(5),
        page: Some(3),
    };
    assert_eq!(q2.per_page(), 5);
    assert_eq!(q2.page(), 3);
    assert_eq!(q2.offset(), 10);
}

#[test]
async fn page_query_paginated_from_data() {
    let q = PageQuery::<10> {
        per_page: Some(5),
        page: Some(2),
    };
    let paginated: Paginated<Vec<i32>> = Paginated::from_data(q, 12, vec![1, 2]);
    assert_eq!(paginated.page, 2);
    assert_eq!(paginated.per_page, 5);
    assert_eq!(paginated.pages, 3);
    assert_eq!(paginated.data, vec![1, 2]);
}

#[actix_web::test]
async fn cache_control_inserts_default_no_store() {
    let app = test::init_service(
        App::new()
            .wrap(CacheController::default_no_store())
            .route("/", web::get().to(|| async { HttpResponse::Ok().finish() })),
    )
    .await;

    let resp = test::call_service(&app, test::TestRequest::get().uri("/").to_request()).await;
    let header_val = resp
        .headers()
        .get(header::CACHE_CONTROL)
        .unwrap()
        .to_str()
        .unwrap();
    assert!(header_val.contains("no-cache"));
    assert!(header_val.contains("no-store"));
}

#[actix_web::test]
async fn cache_control_replaces_existing_header_when_requested() {
    let app = test::init_service(
        App::new()
            .wrap(CacheController::public_with_max_age(60))
            .route(
                "/",
                web::get().to(|| async {
                    HttpResponse::Ok()
                        .insert_header((header::CACHE_CONTROL, "private"))
                        .finish()
                }),
            ),
    )
    .await;

    let resp = test::call_service(&app, test::TestRequest::get().uri("/").to_request()).await;
    let header_val = resp
        .headers()
        .get(header::CACHE_CONTROL)
        .unwrap()
        .to_str()
        .unwrap();
    assert_eq!(header_val, "public, max-age=60");
}

#[test]
async fn error_handler_api_error_new_and_display() {
    let err = ApiError::new(404, "Not found");
    assert_eq!(err.error_status_code, 404);
    assert_eq!(err.to_string(), "Not found");
}

#[actix_web::test]
async fn error_handler_client_error_response_preserves_message() {
    let app = test::init_service(App::new().route(
        "/",
        web::get().to(|| async {
            Err::<HttpResponse, ApiError>(ApiError::new(400, "bad request"))
        }),
    ))
    .await;

    let resp = test::call_service(&app, test::TestRequest::get().uri("/").to_request()).await;
    assert_error_response(resp, 400, Some("bad request")).await;
}

#[actix_web::test]
async fn error_handler_server_error_response_masks_message() {
    let app = test::init_service(App::new().route(
        "/",
        web::get().to(|| async { Err::<HttpResponse, ApiError>(ApiError::new(500, "details")) }),
    ))
    .await;

    let resp = test::call_service(&app, test::TestRequest::get().uri("/").to_request()).await;
    assert_error_response(resp, 500, Some("Internal server error")).await;
}
