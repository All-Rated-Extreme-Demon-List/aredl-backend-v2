use crate::aredl::levels::id_resolver::resolve_level_id;
use crate::aredl::records::PublicRecordResolved;
use crate::aredl::records::Record;
use crate::cache_control::CacheController;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "List records",
    description = "List all of this levels records",
    tag = "AREDL - Levels",
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, body = [PublicRecordResolved])
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    level_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let records =
        web::block(move || PublicRecordResolved::find_all_by_level(db, level_id)).await??;
    Ok(HttpResponse::Ok().json(records))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL - Levels (Records)", description = "Endpoints for fetching and managing records of a specific level")
    ),
    components(
        schemas(
            Record,
            PublicRecordResolved,
        )
    ),
    paths(
        find_all,
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/{level_id}/records").service(find_all));
}
