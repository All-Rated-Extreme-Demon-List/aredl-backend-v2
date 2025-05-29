use crate::arepl::levels::history::{HistoryLevelFull, HistoryLevelResponse};
use crate::arepl::levels::id_resolver::resolve_level_id;
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
    tag = "AREDL (P) - Levels",
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
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let entries = web::block(move || HistoryLevelFull::find(db, level_id)).await??;
    // map history
    let response = entries
        .into_iter()
        .map(|data| HistoryLevelResponse::from_data(&data, level_id))
        .collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(response))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/{level_id}/history").service(find));
}

#[derive(OpenApi)]
#[openapi(components(schemas(HistoryLevelResponse,)), paths(find))]
pub struct ApiDoc;
