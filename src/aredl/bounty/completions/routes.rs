use crate::aredl::bounty::completions::ResolvedCompletedBounty;
use crate::cache_control::CacheController;
use crate::error_handler::ApiError;
use crate::{app_data::db::DbAppState, aredl::bounty::Bounty};
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "Bounty Completions",
    description = "Get the list of completions for a specific bounty",
    tag = "AREDL - Bounty Board",
    responses(
        (status = 200, body = [Vec<ResolvedCompletedBounty>])
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(300)")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    bounty_id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        Bounty::find_completions_from_id(&mut db.connection()?, bounty_id.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(components(schemas(ResolvedCompletedBounty)), paths(list))]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/{bounty_id}/completions").service(list));
}
