use crate::auth::{Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::error_handler::{ApiError, ErrorResponse};
use crate::list::bounty::completions::ResolvedCompletedBounty;
use crate::list::{bounty::routes::BountyPath, List};
use crate::{app_data::db::DbAppState, list::bounty::Bounty};
use actix_web::{get, post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "Bounty Completions",
    description = "Get the list of completions for a specific bounty",
    tag = "List - Bounty Board",
    responses(
        (status = 200, body = [ResolvedCompletedBounty]),
    ),
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("bounty_id" = Uuid, Path, description = "Internal bounty UUID"),
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(300)")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<BountyPath>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        Bounty::find_completions_from_id(&mut db.connection()?, path.list, path.bounty_id)
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Synchronize Bounty Completions",
    description = "Adds any missing completions for this bounty based on existing records. ",
    tag = "List - Bounty Board",
    responses(
        (status = 204),
        (status = 404, description = "Bounty not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["BountyManage"])),
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("bounty_id" = Uuid, Path, description = "Internal bounty UUID"),
    ),
)]
#[post("/sync", wrap = "UserAuth::require(Permission::BountyManage)")]
async fn sync_completions(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<BountyPath>,
) -> Result<HttpResponse, ApiError> {
    web::block(move || {
        let conn = &mut db.connection()?;
        Bounty::find_by_id(conn, path.list, path.bounty_id)?.sync_completions(conn, path.list)
    })
    .await??;
    Ok(HttpResponse::NoContent().finish())
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
