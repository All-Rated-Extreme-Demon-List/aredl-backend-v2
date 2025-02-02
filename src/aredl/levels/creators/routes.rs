use std::sync::Arc;
use actix_web::{delete, get, HttpResponse, patch, post, web};
use uuid::Uuid;
use utoipa::OpenApi;
use crate::users::BaseUser;
use crate::aredl::levels::id_resolver::resolve_level_id;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::auth::{UserAuth, Permission};

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
#[get("")]
async fn find_all(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let creators = web::block(
        move || BaseUser::find_all_creators(db, level_id)
    ).await??;
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
#[post("", wrap="UserAuth::require(Permission::LevelModify)")]
async fn set(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>, creators: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let creators = web::block(
        move || BaseUser::set_all_creators(db, level_id, creators.into_inner())
    ).await??;
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
#[patch("", wrap="UserAuth::require(Permission::LevelModify)")]
async fn add(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>, creators: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let creators = web::block(
        move || BaseUser::add_all_creators(db, level_id, creators.into_inner())
    ).await??;
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
#[delete("", wrap="UserAuth::require(Permission::LevelModify)")]
async fn delete(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>, creators: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let creators = web::block(
        move || BaseUser::delete_all_creators(db, level_id, creators.into_inner())
    ).await??;
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
            .service(delete)
    );
}