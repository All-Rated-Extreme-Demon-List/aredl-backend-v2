use std::sync::Arc;
use actix_web::{get, HttpResponse, web};
use utoipa::OpenApi;
use crate::aredl::levels::id_resolver::resolve_level_id;
use crate::aredl::packs::PackWithTierResolved;
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[utoipa::path(
    get,
    summary = "Get packs",
    description = "List all of the packs this level is in",
    tag = "AREDL - Levels",
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, body = [PackWithTierResolved])
    ),
)]
#[get("")]
async fn find_all(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let packs = web::block(
        move || PackWithTierResolved::find_all(db, level_id)
    ).await??;
    Ok(HttpResponse::Ok().json(packs))
}

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            PackWithTierResolved,
        )
    ),
    paths(
        find_all
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{level_id}/packs")
            .service(find_all)
    );
}