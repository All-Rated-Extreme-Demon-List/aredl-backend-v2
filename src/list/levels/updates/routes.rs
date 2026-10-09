use crate::list::{levels::routes::LevelPath, List};
use crate::{
    app_data::db::DbAppState,
    auth::{Permission, UserAuth},
    error_handler::{ApiError, ErrorResponse},
    list::levels::{
        id_resolver::resolve_level_id,
        updates::{
            LevelUpdateEntry, LevelUpdateEntryPost, LevelUpdateEntryQueryOptions,
            LevelUpdateEntryUpdate, LevelUpdateType,
        },
    },
    CacheController,
};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "List Updates",
    description = "List all updates for a level",
    tag = "List - Levels (Updates)",
    responses(
        (status = 200, body = Vec<LevelUpdateEntry>),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
    ),
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)"),
        ("type_filter" = Option<LevelUpdateType>, Query, description = "The type of update to filter by."),
    )
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    query: web::Query<LevelUpdateEntryQueryOptions>,
    path: web::Path<LevelPath>,
) -> Result<HttpResponse, ApiError> {
    let updates = web::block(move || {
        LevelUpdateEntry::find_all_level(
            &mut db.connection()?,
            &query.into_inner(),
            path.list,
            &path.level_id,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(updates))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add Update",
    description = "Add an update to a level",
    tag = "List - Levels (Updates)",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 201, body = LevelUpdateEntry),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["LevelUpdatesModify"])),
)]
#[post("", wrap = "UserAuth::require(Permission::LevelUpdatesModify)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<LevelUpdateEntryPost>,
    path: web::Path<LevelPath>,
) -> Result<HttpResponse, ApiError> {
    let created = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
        LevelUpdateEntry::create(conn, body.into_inner(), path.list, level_id)
    })
    .await??;
    Ok(HttpResponse::Created().json(created))
}

#[derive(serde::Deserialize)]
struct UpdatePath {
    list: List,
    level_id: String,
    update_id: Uuid,
}

#[utoipa::path(
    patch,
    summary = "[Staff]Update Update",
    description = "Update a level update's info",
    tag = "List - Levels (Updates)",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)"),
        ("update_id" = Uuid, Path, description = "The internal ID of this update")
    ),
    responses(
        (status = 200, body = LevelUpdateEntry),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level update not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["LevelUpdatesModify"])),
)]
#[patch(
    "/{update_id}",
    wrap = "UserAuth::require(Permission::LevelUpdatesModify)"
)]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<LevelUpdateEntryUpdate>,
    path: web::Path<UpdatePath>,
) -> Result<HttpResponse, ApiError> {
    let updated = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
        LevelUpdateEntry::update(
            conn,
            body.into_inner(),
            path.list,
            level_id,
            &path.update_id,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(updated))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete Update",
    description = "Deletes a level update",
    tag = "List - Levels (Updates)",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)"),
        ("update_id" = Uuid, Path, description = "The internal ID of this update")
    ),
    responses(
        (status = 204),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse),
    ),
    security(("bearer_token" = ["LevelUpdatesModify"])),
)]
#[delete(
    "/{update_id}",
    wrap = "UserAuth::require(Permission::LevelUpdatesModify)"
)]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<UpdatePath>,
) -> Result<HttpResponse, ApiError> {
    web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
        LevelUpdateEntry::delete(conn, path.list, level_id, &path.update_id)
    })
    .await??;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(OpenApi)]
#[openapi(
    tags((
        name = "List - Levels (Updates)",
        description = "Endpoints for fetching and managing level updates on the AREDL",
    )),
    components(schemas(
        LevelUpdateEntry,
        LevelUpdateEntryPost,
        LevelUpdateEntryUpdate,
    )),
    paths(find_all, create, update, delete)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{level_id}/updates")
            .service(find_all)
            .service(create)
            .service(update)
            .service(delete),
    );
}
