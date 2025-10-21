use crate::aredl::levels::id_resolver::resolve_level_id;
use crate::auth::{Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
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
    tag = "AREDL - Levels (Creators)",
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, body = [BaseUser])
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    level_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let creators = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, level_id.into_inner().as_str())?;
        BaseUser::aredl_find_all_creators(conn, level_id)
    })
    .await??;
    Ok(HttpResponse::Ok().json(creators))
}

#[utoipa::path(
    post,
    summary = "[Staff]Set all creators",
    description = "Change all the creators of a level to the given list",
    tag = "AREDL - Levels (Creators)",
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, description = "Creators set successfully", body = [BaseUser])
    ),
    security(
        ("access_token" = ["LevelModify"]),
        ("api_key" = ["LevelModify"]),
    )
)]
#[post("", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn set(
    db: web::Data<Arc<DbAppState>>,
    level_id: web::Path<String>,
    creators: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&creators));
    let creators = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, level_id.into_inner().as_str())?;
        BaseUser::aredl_set_all_creators(conn, level_id, creators.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(creators))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Add creators",
    description = "Add the given creators to this level's creators list",
    tag = "AREDL - Levels (Creators)",
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, description = "Creators added successfully", body = [BaseUser])
    ),
    security(
        ("access_token" = ["LevelModify"]),
        ("api_key" = ["LevelModify"]),
    )
)]
#[patch("", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn add(
    db: web::Data<Arc<DbAppState>>,
    level_id: web::Path<String>,
    creators: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&creators));
    let creators = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, level_id.into_inner().as_str())?;
        BaseUser::aredl_add_all_creators(conn, level_id, creators.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(creators))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Remove creators",
    description = "Remove the given creators from this level's creators list",
    tag = "AREDL - Levels (Creators)",
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, description = "Creators removed successfully", body = [BaseUser])
    ),
    security(
        ("access_token" = ["LevelModify"]),
        ("api_key" = ["LevelModify"]),
    )
)]
#[delete("", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    level_id: web::Path<String>,
    creators: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&creators));
    let creators = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, level_id.into_inner().as_str())?;
        BaseUser::aredl_delete_all_creators(conn, level_id, creators.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(creators))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL - Levels (Creators)", description = "Endpoints for fetching and managing the creators list of a specific level"),
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
