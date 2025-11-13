use crate::{
    aredl::records::statistics::{total_records, ResolvedLevelTotalRecordsRow},
    cache_control::CacheController,
    app_data::db::DbAppState,
    error_handler::ApiError,
};
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "[Staff]Total records",
    description = "List levels ranked by number of records, as well as total records.",
    tag = "AREDL - Records",
    responses((status = 200, body = [ResolvedLevelTotalRecordsRow])),
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
pub async fn total(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let data = web::block(move || total_records(&mut db.connection()?)).await??;
    Ok(HttpResponse::Ok().json(data))
}

#[derive(OpenApi)]
#[openapi(components(schemas(ResolvedLevelTotalRecordsRow)), paths(total))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/statistics").service(total));
}
