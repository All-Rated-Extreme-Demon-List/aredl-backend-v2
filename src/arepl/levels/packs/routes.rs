use crate::arepl::levels::id_resolver::resolve_level_id;
use crate::arepl::packs::PackWithTierResolved;
use crate::cache_control::CacheController;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "Get packs",
    description = "List all of the packs this level is in",
    tag = "AREDL (P) - Levels",
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, body = [PackWithTierResolved])
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    level_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let packs = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, level_id.into_inner().as_str())?;
        PackWithTierResolved::find_all(conn, level_id)
    })
    .await??;
    Ok(HttpResponse::Ok().json(packs))
}

#[derive(OpenApi)]
#[openapi(components(schemas(PackWithTierResolved,)), paths(find_all))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/{level_id}/packs").service(find_all));
}
