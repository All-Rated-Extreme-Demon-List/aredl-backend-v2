use crate::app_data::db::DbAppState;
use crate::auth::{Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::error_handler::{ApiError, ErrorResponse};
use crate::list::levels::id_resolver::resolve_level_id;
use crate::list::levels::routes::LevelPath;
use crate::list::List;
use crate::users::BaseUser;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "List all creators",
    description = "List all creators of a level",
    tag = "List - Levels (Creators)",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, body = [BaseUser]),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse)
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<LevelPath>,
) -> Result<HttpResponse, ApiError> {
    let creators = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
        BaseUser::find_all_creators(conn, path.list, level_id)
    })
    .await??;
    Ok(HttpResponse::Ok().json(creators))
}

#[utoipa::path(
    post,
    summary = "[Staff]Set all creators",
    description = "Change all the creators of a level to the given list",
    tag = "List - Levels (Creators)",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, description = "Creators set successfully", body = [Uuid]),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["LevelModify"])),
)]
#[post("", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn set(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<LevelPath>,
    creators: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&creators));
    let creators = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
        BaseUser::set_all_creators(conn, path.list, level_id, &creators.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(creators))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Add creators",
    description = "Add the given creators to this level's creators list",
    tag = "List - Levels (Creators)",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, description = "Creators added successfully", body = [Uuid]),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["LevelModify"])),
)]
#[patch("", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn add(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<LevelPath>,
    creators: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&creators));
    let creators = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
        BaseUser::add_all_creators(conn, path.list, level_id, &creators.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(creators))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Remove creators",
    description = "Remove the given creators from this level's creators list",
    tag = "List - Levels (Creators)",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level ID (Can be internal UUID, list position, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, description = "Creators removed successfully", body = [Uuid]),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["LevelModify"])),
)]
#[delete("", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<LevelPath>,
    creators: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&creators));
    let creators = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
        BaseUser::delete_all_creators(conn, path.list, level_id, &creators.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(creators))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "List - Levels (Creators)", description = "Endpoints for fetching and managing the creators list of a specific level"),
    ),
    paths(
        find_all,
        add,
        set,
        delete
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{level_id}/creators")
            .service(find_all)
            .service(add)
            .service(set)
            .service(delete),
    );
}
