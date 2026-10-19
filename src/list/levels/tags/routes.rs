use super::model::LevelTag;
use crate::app_data::db::DbAppState;
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::error_handler::{ApiError, ErrorResponse};
use crate::list::levels::{id_resolver::resolve_level_id, routes::LevelPath};
use crate::list::tags::Tag;
use crate::list::List;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "List level tags",
    description = "List all tags assigned to a level.",
    tag = "List - Levels (Tags)",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level UUID, list position, or GD ID (with the _2p suffix for the two-player variant)")
    ),
    responses(
        (status = 200, body = [Tag]),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse)
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<LevelPath>,
) -> Result<HttpResponse, ApiError> {
    let tags = db
        .run(move |conn| {
            let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
            Tag::find_all_level(conn, path.list, level_id)
        })
        .await?;
    Ok(HttpResponse::Ok().json(tags))
}

#[utoipa::path(
    post,
    summary = "[Staff]Set all level tags",
    description = "Replace all tags currently assigned to the level with the given tag UUIDs.",
    tag = "List - Levels (Tags)",
    request_body = [Uuid],
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level UUID, list position, or GD ID (with the _2p suffix for the two-player variant)")
    ),
    responses(
        (status = 200, description = "Current tag UUIDs assigned to the level", body = [Uuid]),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse),
        (status = 422, description = "Tags must exist for the selected list", body = ErrorResponse)
    ),
    security(("bearer_token" = ["LevelTagsModify"])),
)]
#[post("", wrap = "UserAuth::require(Permission::LevelTagsModify)")]
async fn set(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<LevelPath>,
    tags: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&tags));
    let tags = db
        .run(move |conn| {
            let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
            LevelTag::set_all_level(
                conn,
                path.list,
                level_id,
                &tags.into_inner(),
                &authenticated,
            )
        })
        .await?;
    Ok(HttpResponse::Ok().json(tags))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Add level tags",
    description = "Add the given tag UUIDs to the level.",
    tag = "List - Levels (Tags)",
    request_body = [Uuid],
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level UUID, list position, or GD ID (with the _2p suffix for the two-player variant)")
    ),
    responses(
        (status = 200, description = "Current tag UUIDs assigned to the level", body = [Uuid]),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse),
        (status = 422, description = "Tags must exist in the selected list", body = ErrorResponse)
    ),
    security(("bearer_token" = ["LevelTagsModify"])),
)]
#[patch("", wrap = "UserAuth::require(Permission::LevelTagsModify)")]
async fn add(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<LevelPath>,
    tags: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&tags));
    let tags = db
        .run(move |conn| {
            let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
            LevelTag::add_all_level(
                conn,
                path.list,
                level_id,
                &tags.into_inner(),
                &authenticated,
            )
        })
        .await?;
    Ok(HttpResponse::Ok().json(tags))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Remove level tags",
    description = "Remove the given tag UUIDs from the level.",
    tag = "List - Levels (Tags)",
    request_body = [Uuid],
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("level_id" = String, Path, description = "Level UUID, list position, or GD ID (with the _2p suffix for the two-player variant)")
    ),
    responses(
        (status = 200, description = "Current tag UUIDs assigned to the level", body = [Uuid]),
        (status = 400, description = "Invalid level ID", body = ErrorResponse),
        (status = 404, description = "Level not found", body = ErrorResponse),
    ),
    security(("bearer_token" = ["LevelTagsModify"])),
)]
#[delete("", wrap = "UserAuth::require(Permission::LevelTagsModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<LevelPath>,
    tags: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&tags));
    let tags = db
        .run(move |conn| {
            let level_id = resolve_level_id(conn, path.list, &path.level_id)?;
            LevelTag::delete_all_level(
                conn,
                path.list,
                level_id,
                &tags.into_inner(),
                &authenticated,
            )
        })
        .await?;
    Ok(HttpResponse::Ok().json(tags))
}

#[derive(OpenApi)]
#[openapi(
    tags((name = "List - Levels (Tags)", description = "Endpoints for fetching and managing a level's tags")),
    paths(find_all, set, add, delete)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{level_id}/tags")
            .service(find_all)
            .service(set)
            .service(add)
            .service(delete),
    );
}
