use actix_web::web;
use actix_web::{get, HttpResponse};
use diesel::RunQueryDsl;
use std::sync::Arc;
use utoipa::OpenApi;

use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[utoipa::path(
    get,
    responses(
        (status = 200, description = "API and DB healthy"),
        (status = 503, description = "Service unavailable"),
    ),
    tag = "Health"
)]
#[get("")]
async fn healthz(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        diesel::sql_query("SELECT 1")
            .execute(&mut db.connection()?)
            .map_err(|error| ApiError::new(503, &format!("DB healthcheck failed: {}", error)))
    })
    .await;

    match result {
        Ok(Ok(_)) => Ok(HttpResponse::Ok().finish()),
        _ => Err(ApiError::new(503, "Service unavailable")),
    }
}

#[derive(OpenApi)]
#[openapi(paths(healthz))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/health").service(healthz));
}
