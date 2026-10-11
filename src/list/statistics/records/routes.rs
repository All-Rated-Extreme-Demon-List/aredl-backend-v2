use crate::list::List;
use crate::{
    app_data::db::DbAppState,
    cache_control::CacheController,
    error_handler::ApiError,
    list::statistics::records::{total_records, ResolvedLevelTotalRecordsRow},
};
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "Total records",
    description = "List levels ranked by number of records, as well as total records and verifications.",
    tag = "List - Statistics",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
    ),
    responses(
        (status = 200, body = [ResolvedLevelTotalRecordsRow]),
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
pub async fn total(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
) -> Result<HttpResponse, ApiError> {
    let data = web::block(move || total_records(&mut db.connection()?, *list)).await??;
    Ok(HttpResponse::Ok().json(data))
}

#[derive(OpenApi)]
#[openapi(components(schemas(ResolvedLevelTotalRecordsRow)), paths(total))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/records").service(total));
}
