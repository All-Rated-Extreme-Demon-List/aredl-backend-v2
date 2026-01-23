use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use utoipa::OpenApi;

use crate::{
    app_data::db::DbAppState,
    aredl::statistics::submissions::{daily, total_submissions, ResolvedQueueLevelSubmissionsRow},
    cache_control::CacheController,
    error_handler::ApiError,
};

#[utoipa::path(
    get,
    summary = "[Staff]Total submissions",
    description = "List levels ranked by number of submissions and part of the current queue, as well as total submissions.",
    tag = "AREDL - Statistics",
    responses((status = 200, body = [ResolvedQueueLevelSubmissionsRow])),
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
pub async fn total(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let data = web::block(move || total_submissions(&mut db.connection()?)).await??;
    Ok(HttpResponse::Ok().json(data))
}

#[derive(OpenApi)]
#[openapi(
	paths(total),
	nest(
        (path = "/daily", api=daily::ApiDoc),
    ),
	components(schemas(ResolvedQueueLevelSubmissionsRow)),
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/submissions")
            .configure(daily::init_routes)
            .service(total),
    );
}
