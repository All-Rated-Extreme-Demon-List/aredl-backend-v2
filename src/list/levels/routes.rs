use crate::app_data::db::DbAppState;
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::error_handler::{ApiError, ErrorResponse};
use crate::list::levels::history::HistoryLevelFull;
use crate::list::levels::{
    creators, custom_copies, history, id_resolver::resolve_level_id, packs, records, Level,
    LevelPlace, LevelUpdate, LevelWithUserCompletionStatus, ResolvedLevel,
};
use crate::list::levels::{notes, updates, LevelQueryOptions};
use crate::list::List;
use crate::scheduled::refresh_level_data;
use actix_web::{get, patch, post, web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;

#[derive(Deserialize)]
pub struct LevelPath {
    pub list: List,
    pub level_id: String,
}

#[utoipa::path(
    get,
    summary = "[AuthPublic]List all levels",
    description = "List all the levels on the list",
    tag = "List - Levels",
    params(
		("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("exclude_legacy" = Option<bool>, Query, description = "Whether levels on the legacy list should be excluded"),
        ("exclude_pending" = Option<bool>, Query, description = "Whether pending levels should be excluded. Defaults to true"),
        ("exclude_removed" = Option<bool>, Query, description = "Whether removed levels should be excluded. Defaults to true"),
        ("at" = Option<DateTime<Utc>>, Query, description = "Return the state of the list at the provided timestamp"),
    ),
    responses(
        (status = 200, body = [LevelWithUserCompletionStatus]),
    ),
    security((), ("bearer_token" = [])),
)]
#[get(
    "",
    wrap = "UserAuth::load()",
    wrap = "CacheController::auth_public_with_max_age(900)"
)]
async fn list_all(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    query: web::Query<LevelQueryOptions>,
    authenticated: Option<Authenticated>,
) -> Result<HttpResponse, ApiError> {
    let levels = web::block(move || {
        LevelWithUserCompletionStatus::find_all(
            &mut db.connection()?,
            *list,
            &query.into_inner(),
            authenticated.map(|user| user.user_id),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(levels))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add level",
    description = "Place a new level on the list",
    tag = "List - Levels",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
    ),
    responses(
        (status = 422, description = "A position is required for MainList/Legacy levels and must be valid", body = ErrorResponse),
        (status = 201, description = "Level added successfully", body = Level),
    ),
    security(("bearer_token" = ["LevelModify"])),
)]
#[post("", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    level: web::Json<LevelPlace>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&level));
    let level = web::block(move || Level::create(&mut db.connection()?, *list, level.into_inner()))
        .await??;
    Ok(HttpResponse::Created().json(level))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit level",
    description = "Edit the base information of a level",
    tag = "List - Levels",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id", description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 422, description = "A position is required for MainList/Legacy levels and must be valid", body = ErrorResponse),
        (status = 200, description = "Level edited successfully", body = Level),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["LevelModify"])),
)]
#[patch("/{level_id}", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<LevelPath>,
    level: web::Json<LevelUpdate>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&level));
    let level = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
        Level::update(conn, path.list, level_id, level.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(level))
}

#[utoipa::path(
    post,
    summary = "[Staff]Refresh GDDL",
    description = "Forces a refresh of the level's GDDL tier.",
    tag = "List - Levels",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 204, description = "GDDL tier refreshed"),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse),
        (status = 502, description = "GDDL request failed", body = ErrorResponse),
    ),
    security(("bearer_token" = ["MaintenanceRun"])),
)]
#[post(
    "/{level_id}/gddl/refresh",
    wrap = "UserAuth::require(Permission::MaintenanceRun)"
)]
async fn refresh_gddl(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<LevelPath>,
) -> Result<HttpResponse, ApiError> {
    let list = path.list;
    let db_level = db.get_ref().clone();
    let id =
        web::block(move || resolve_level_id(&mut db_level.connection()?, list, &path.level_id))
            .await??;
    refresh_level_data::refresh_gddl(db.get_ref().clone(), list, id).await?;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    post,
    summary = "[Staff]Rebuild position history",
    description = "Regenerates the position history full table based on the current changelog data",
    tag = "List - Levels",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
    ),
    responses(
        (status = 204),
    ),
    security(("bearer_token" = ["MaintenanceRun"])),
)]
#[post(
    "/history/rebuild",
    wrap = "UserAuth::require(Permission::MaintenanceRun)"
)]
async fn rebuild_history(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
) -> Result<HttpResponse, ApiError> {
    web::block(move || HistoryLevelFull::rebuild(&mut db.connection()?, *list)).await??;
    Ok(HttpResponse::NoContent().finish())
}

#[utoipa::path(
    get,
    summary = "Get level details",
    description = "Get more detailed information about a level",
    tag = "List - Levels",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id", description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, body = ResolvedLevel),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse)
    ),
)]
#[get("/{level_id}", wrap = "CacheController::public_with_max_age(900)")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<LevelPath>,
) -> Result<HttpResponse, ApiError> {
    let level = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
        ResolvedLevel::find(conn, path.list, level_id)
    })
    .await??;
    Ok(HttpResponse::Ok().json(level))
}

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/{level_id}/creators", api = creators::ApiDoc),
        (path = "/{level_id}/history", api = history::ApiDoc),
        (path = "/{level_id}/records", api = records::ApiDoc),
        (path = "/{level_id}/packs", api = packs::ApiDoc),
        (path = "/{level_id}/custom-copies", api = custom_copies::ApiDoc),
        (path = "/{level_id}/notes", api = notes::ApiDoc),
        (path = "/{level_id}/updates", api = updates::ApiDoc),
    ),
    tags(
        (name = "List - Levels", description="Endpoints for fetching and managing levels on the AREDL")
    ),
    components(
        schemas(
            Level,
            LevelWithUserCompletionStatus,
        )
    ),
    paths(
        list_all,
        create,
        update,
        find,
        refresh_gddl,
        rebuild_history,
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/levels")
            .configure(history::init_routes)
            .configure(packs::init_routes)
            .configure(records::init_routes)
            .configure(creators::init_routes)
            .configure(custom_copies::init_routes)
            .configure(notes::init_routes)
            .configure(updates::init_routes)
            .service(list_all)
            .service(create)
            .service(update)
            .service(refresh_gddl)
            .service(rebuild_history)
            .service(find),
    );
}
