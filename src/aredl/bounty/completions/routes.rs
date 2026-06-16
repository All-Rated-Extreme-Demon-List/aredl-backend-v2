use crate::aredl::bounty::completions::ResolvedCompletedBounty;
use crate::auth::{Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::error_handler::ApiError;
use crate::{app_data::db::DbAppState, aredl::bounty::Bounty};
use actix_web::{HttpResponse, get, post, web};
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

#[utoipa::path(
    post,
    summary = "Synchronize Bounty Completions",
    description = "Adds any missing completions for this bounty based on existing records. ",
    tag = "AREDL - Bounty Board",
    responses(
        (status = 200)
    ),
)]
#[post("/sync", wrap = "UserAuth::require(Permission::BountyManage)")]
async fn sync_completions(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    web::block(move || {
        let conn = &mut db.connection()?;
        Bounty::find_by_id(conn, id.into_inner())?.sync_completions(conn)
    })
    .await??;
    Ok(HttpResponse::Ok().json(()))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(ResolvedCompletedBounty)),
    paths(list, sync_completions)
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{bounty_id}/completions")
            .service(list)
            .service(sync_completions),
    );
}
