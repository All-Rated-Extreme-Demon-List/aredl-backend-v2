use std::sync::Arc;
use actix_web::{get, HttpResponse, web};
use utoipa::OpenApi;
use crate::aredl::levels::history::{HistoryLevelResponse, HistoryLevelFull};
use crate::aredl::levels::id_resolver::resolve_level_id;
use crate::db::DbAppState;
use crate::error_handler::ApiError;

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
#[get("")]
async fn find(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let entries = web::block(move || HistoryLevelFull::find(db, level_id)).await??;
    // map history
    let response = entries
        .into_iter()
        .map(|data| HistoryLevelResponse::from_data(&data, level_id)).collect::<Vec<_>>();
    Ok(HttpResponse::Ok().json(response))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{level_id}/history")
            .service(find)
    );
}

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            HistoryLevelResponse,
        )
    ),
    paths(
        find
    )
)]
pub struct ApiDoc;
