use crate::aredl::levels::id_resolver::resolve_level_id;
use crate::aredl::levels::{
    creators, history, ldms, packs, records, Level, LevelPlace, LevelUpdate, ResolvedLevel,
};
use crate::auth::{Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::app_data::db::DbAppState;
use crate::error_handler::ApiError;
use actix_web::{get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;

#[derive(serde::Deserialize)]
struct LevelQueryOptions {
    exclude_legacy: Option<bool>,
}

#[utoipa::path(
    get,
    summary = "List all levels",
    description = "List all the levels on the list",
    tag = "AREDL - Levels",
    params(
        ("exclude_legacy" = Option<bool>, Query, description = "Whether levels on the legacy list should be excluded"),
    ),
    responses((status = 200, body = [Level]))
)]
#[get("", wrap = "CacheController::public_with_max_age(900)")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    query: web::Query<LevelQueryOptions>,
) -> Result<HttpResponse, ApiError> {
    let levels =
        web::block(move || Level::find_all(&mut db.connection()?, query.exclude_legacy)).await??;
    Ok(HttpResponse::Ok().json(levels))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add level",
    description = "Place a new level on the list",
    tag = "AREDL - Levels",
    responses((status = 200, description = "Level added successfully", body = Level)),
    security(("access_token" = ["LevelModify"]), ("api_key" = ["LevelModify"]))
)]
#[post("", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    level: web::Json<LevelPlace>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&level));
    let level =
        web::block(move || Level::create(&mut db.connection()?, level.into_inner())).await??;
    Ok(HttpResponse::Ok().json(level))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit level",
    description = "Edit the base information of a level",
    tag = "AREDL - Levels",
    params((
        "level_id",
        description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)",
    )),
    responses((status = 200, description = "Level edited successfully", body = Level)),
    security(("access_token" = ["LevelModify"]), ("api_key" = ["LevelModify"]))
)]
#[patch("/{level_id}", wrap = "UserAuth::require(Permission::LevelModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    level_id: web::Path<String>,
    level: web::Json<LevelUpdate>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&level));
    let level = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, level_id.into_inner().as_str())?;
        Level::update(conn, level_id, level.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(level))
}

#[utoipa::path(
    get,
    summary = "Get level details",
    description = "Get more detailed information about a level",
    tag = "AREDL - Levels",
    params((
        "level_id",
        description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)",
    )),
    responses((status = 200, body = ResolvedLevel))
)]
#[get("/{level_id}", wrap = "CacheController::public_with_max_age(900)")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    level_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let level = web::block(move || {
        let conn = &mut db.connection()?;
        let level_id = resolve_level_id(conn, level_id.into_inner().as_str())?;
        ResolvedLevel::find(conn, level_id)
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
        (path = "/ldms", api = ldms::ApiDoc)
    ),
    tags((
        name = "AREDL - Levels",
        description = "Endpoints for fetching and managing levels on the AREDL",
    )),
    components(schemas(Level)),
    paths(list, create, update, find)
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/levels")
            .configure(history::init_routes)
            .configure(packs::init_routes)
            .configure(records::init_routes)
            .configure(ldms::init_routes)
            .configure(creators::init_routes)
            .service(list)
            .service(create)
            .service(update)
            .service(find),
    );
}
