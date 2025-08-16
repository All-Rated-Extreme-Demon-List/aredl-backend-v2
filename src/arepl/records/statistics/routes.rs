use crate::{
    arepl::records::statistics::{total_records, ResolvedLevelTotalRecordsRow},
    db::DbAppState,
    error_handler::ApiError,
};
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "[Staff]Total records",
    description = "List levels ranked by number of records, as well as total records.",
    tag = "AREDL (P) - Records",
    responses((status = 200, body = [ResolvedLevelTotalRecordsRow])),
)]
#[get("")]
pub async fn total(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let data = web::block(move || total_records(db)).await??;
    Ok(HttpResponse::Ok().json(data))
}

#[derive(OpenApi)]
#[openapi(components(schemas(ResolvedLevelTotalRecordsRow)), paths(total))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/statistics").service(total));
}
