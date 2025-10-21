use crate::aredl::levels::history::{HistoryLevelFull, HistoryLevelResponse};
use crate::aredl::levels::id_resolver::resolve_level_id;
use crate::cache_control::CacheController;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "Get history",
    description = "Get all of this level's placement history",
    tag = "AREDL - Levels",
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, body = [HistoryLevelResponse])
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    level_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let response = web::block(move || -> Result<Vec<HistoryLevelResponse>, ApiError> {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, level_id.into_inner().as_str())?;
        let entries = HistoryLevelFull::find(conn, level_id);

        // map history
        let mapped = entries
            .into_iter()
            .flatten()
            .map(|data| HistoryLevelResponse::from_data(&data, level_id))
            .collect::<Vec<_>>();

        Ok(mapped)
    })
    .await??;

    Ok(HttpResponse::Ok().json(response))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/{level_id}/history").service(find));
}

#[derive(OpenApi)]
#[openapi(components(schemas(HistoryLevelResponse,)), paths(find))]
pub struct ApiDoc;
