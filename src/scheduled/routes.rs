use crate::{
    app_data::db::DbAppState,
    auth::{Permission, UserAuth},
    error_handler::{ApiError, ErrorResponse},
    notifications::WebsocketNotification,
    providers::ProvidersAppState,
    scheduled::{
        data_cleaner::{self, CleanupResult},
        refresh_level_data::{self, LevelDataRefreshResult},
        refresh_matviews::{self, Matview},
        sync_patreon_plus::{self, PatreonPlusSyncResult},
    },
};
use actix_web::{post, web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::broadcast;
use utoipa::OpenApi;

#[derive(Deserialize)]
struct MatviewsQuery {
    view: Matview,
}

#[utoipa::path(
    post,
    summary = "[Staff]Refresh materialized view",
    description = "Refresh a database materialized view (leaderboards or other agregated data)",
    tag = "Scheduled",
    params(
        ("view" = Matview, Query, description = "The view to refresh"),
    ),
    responses(
        (status = 204, description = "Materialized view refreshed"),
        (status = 400, description = "Missing or unknown materialized view", body = ErrorResponse),
    ),
    security(("bearer_token" = ["MaintenanceRun"])),
)]
#[post(
    "/matviews/refresh",
    wrap = "UserAuth::require(Permission::MaintenanceRun)"
)]
async fn matviews(
    db: web::Data<Arc<DbAppState>>,
    query: web::Query<MatviewsQuery>,
) -> Result<HttpResponse, ApiError> {
    refresh_matviews::refresh(db.get_ref().clone(), query.into_inner().view).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    post,
    summary = "[Staff]Sync Patreon",
    description = "Forcefully syncs the current Patreon members list and the AREDL+ members on the site",
    tag = "Scheduled",
    responses(
        (status = 200, body = PatreonPlusSyncResult),
        (status = 400, description = "Patreon auth is not configured", body = ErrorResponse),
        (status = 502, description = "Patreon request failed", body = ErrorResponse),
    ),
    security(("bearer_token" = ["MaintenanceRun"])),
)]
#[post(
    "/patreon/sync",
    wrap = "UserAuth::require(Permission::MaintenanceRun)"
)]
async fn patreon(
    db: web::Data<Arc<DbAppState>>,
    providers: web::Data<Arc<ProvidersAppState>>,
) -> Result<HttpResponse, ApiError> {
    let result = sync_patreon_plus::sync(db.get_ref().clone(), providers.get_ref().clone()).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Refresh EDEL",
    description = "Fetches data from EDEL and updates levels",
    tag = "Scheduled",
    responses(
        (status = 200, body = LevelDataRefreshResult),
        (status = 400, description = "Google auth is not configured", body = ErrorResponse),
        (status = 502, description = "Spreadsheet request failed", body = ErrorResponse),
    ),
    security(("bearer_token" = ["MaintenanceRun"])),
)]
#[post(
    "/level-data/edel/refresh",
    wrap = "UserAuth::require(Permission::MaintenanceRun)"
)]
async fn edel(
    db: web::Data<Arc<DbAppState>>,
    providers: web::Data<Arc<ProvidersAppState>>,
) -> Result<HttpResponse, ApiError> {
    let result =
        refresh_level_data::refresh_edel(db.get_ref().clone(), providers.get_ref().clone()).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Refresh NLW",
    description = "Fetches data from NLW and updates levels.",
    tag = "Scheduled",
    responses(
        (status = 200, body = LevelDataRefreshResult),
        (status = 400, description = "Google auth is not configured", body = ErrorResponse),
        (status = 502, description = "Spreadsheet request failed", body = ErrorResponse),
    ),
    security(("bearer_token" = ["MaintenanceRun"])),
)]
#[post(
    "/level-data/nlw/refresh",
    wrap = "UserAuth::require(Permission::MaintenanceRun)"
)]
async fn nlw(
    db: web::Data<Arc<DbAppState>>,
    providers: web::Data<Arc<ProvidersAppState>>,
) -> Result<HttpResponse, ApiError> {
    let result =
        refresh_level_data::refresh_nlw(db.get_ref().clone(), providers.get_ref().clone()).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Cleanup",
    description = "Deletes old notifications, sets back to Pending submissions that have been claimed for too long, and expire overdue shifts.",
    tag = "Scheduled",
    responses(
        (status = 200, body = CleanupResult),
    ),
    security(("bearer_token" = ["MaintenanceRun"])),
)]
#[post("/cleanup", wrap = "UserAuth::require(Permission::MaintenanceRun)")]
async fn cleanup(
    db: web::Data<Arc<DbAppState>>,
    notify_tx: web::Data<broadcast::Sender<WebsocketNotification>>,
) -> Result<HttpResponse, ApiError> {
    let result = data_cleaner::cleanup(db.get_ref().clone(), notify_tx.get_ref().clone()).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Scheduled", description = "Staff endpoints for manually running scheduled actions"),
    ),
    components(
        schemas(
            Matview,
            PatreonPlusSyncResult,
            LevelDataRefreshResult,
            CleanupResult,
        )
    ),
    paths(
        matviews,
        patreon,
        edel,
        nlw,
        cleanup,
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/scheduled")
            .service(matviews)
            .service(patreon)
            .service(edel)
            .service(nlw)
            .service(cleanup),
    );
}
