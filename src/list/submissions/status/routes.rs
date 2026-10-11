use crate::list::List;
use crate::{
    app_data::db::DbAppState,
    auth::{Authenticated, Permission, UserAuth},
    cache_control::CacheController,
    error_handler::{ApiError, ErrorResponse},
    list::submissions::status::{SubmissionsEnabled, SubmissionsEnabledFull},
};
use actix_web::{get, post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    post,
    summary = "[Staff]Enable submissions",
    description = "Toggle submissions on, allowing users to submit records to the list",
    tag = "List - Submissions",
    params(("list" = List, Path, description = "The selected list (classic / aredl or platformer / arepl)")),
    responses(
        (status = 204),
    ),
    security(("bearer_token" = ["SubmissionStatusManage"])),
)]
#[post(
    "/enable",
    wrap = "UserAuth::require(Permission::SubmissionStatusManage)"
)]
async fn enable(
    list: web::Path<List>,
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    web::block(move || {
        SubmissionsEnabled::enable(&mut db.connection()?, *list, authenticated.user_id)
    })
    .await??;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    post,
    summary = "[Staff]Disable submissions",
    description = "Toggle submissions off, stopping users from submitting records to the list.",
    tag = "List - Submissions",
    params(("list" = List, Path, description = "The selected list (classic / aredl or platformer / arepl)")),
    responses(
        (status = 204),
    ),
    security(("bearer_token" = ["SubmissionStatusManage"])),
)]
#[post(
    "/disable",
    wrap = "UserAuth::require(Permission::SubmissionStatusManage)"
)]
async fn disable(
    list: web::Path<List>,
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    web::block(move || {
        SubmissionsEnabled::disable(&mut db.connection()?, *list, authenticated.user_id)
    })
    .await??;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    summary = "[Staff]Get submission status details",
    description = "Get the status of submissions. In addition to the status, this also returns the moderator who last enabled/disabled submissions, and the timestamp.",
    tag = "List - Submissions",
    params(("list" = List, Path, description = "The selected list (classic / aredl or platformer / arepl)")),
    responses(
        (status = 200, body = SubmissionsEnabledFull),
        (status = 404, description = "No submission status has been registered yet.", body = ErrorResponse)
    ),
    security(("bearer_token" = ["SubmissionStatusManage"])),
)]
#[get(
    "/full",
    wrap = "UserAuth::require(Permission::SubmissionStatusManage)"
)]
async fn get_status_full(
    list: web::Path<List>,
    db: web::Data<Arc<DbAppState>>,
) -> Result<HttpResponse, ApiError> {
    let res = web::block(move || SubmissionsEnabledFull::get_status(&mut db.connection()?, *list))
        .await??;
    Ok(HttpResponse::Ok().json(res))
}

#[utoipa::path(
    get,
    summary = "Get submission status",
    description = "Get the status of submissions. Returns `false` if submissions are closed, and `true` otherwise.",
    tag = "List - Submissions",
    params(("list" = List, Path, description = "The selected list (classic / aredl or platformer / arepl)")),
    responses(
        (status = 200, body = bool, content_type = "application/json"),
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(60)")]
async fn get_status(
    list: web::Path<List>,
    db: web::Data<Arc<DbAppState>>,
) -> Result<HttpResponse, ApiError> {
    let res =
        web::block(move || SubmissionsEnabled::is_enabled(&mut db.connection()?, *list)).await??;
    Ok(HttpResponse::Ok().json(res))
}

#[utoipa::path(
    get,
    summary = "[Staff]Get submission status history",
    description = "Get a log of when submissions were enabled or disabled and by whom.",
    tag = "List - Submissions",
    params(("list" = List, Path, description = "The selected list (classic / aredl or platformer / arepl)")),
    responses(
        (status = 200, body = [SubmissionsEnabledFull]),
    ),
    security(("bearer_token" = ["SubmissionStatusManage"])),
)]
#[get(
    "/history",
    wrap = "UserAuth::require(Permission::SubmissionStatusManage)"
)]
async fn get_history(
    list: web::Path<List>,
    db: web::Data<Arc<DbAppState>>,
) -> Result<HttpResponse, ApiError> {
    let res =
        web::block(move || SubmissionsEnabledFull::get_statuses(&mut db.connection()?, *list))
            .await??;
    Ok(HttpResponse::Ok().json(res))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(SubmissionsEnabled, SubmissionsEnabledFull)),
    paths(get_status, get_status_full, enable, disable, get_history)
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/status")
            .service(get_status)
            .service(get_status_full)
            .service(get_history)
            .service(enable)
            .service(disable),
    );
}
