use crate::app_data::db::DbAppState;
use crate::cache_control::CacheController;
use crate::error_handler::{ApiError, ErrorResponse};
use crate::list::clan::ClanProfileResolved;
use crate::list::levels::records::LevelResolvedRecordExtended;
use crate::list::List;
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

#[derive(Deserialize)]
struct ClanPath {
    list: List,
    clan_id: Uuid,
}

#[derive(Deserialize)]
struct ClanLevelPath {
    list: List,
    clan_id: Uuid,
    level_id: Uuid,
}

#[utoipa::path(
    get,
    summary = "Clan",
    description = "Get a clan's profile for the selected list",
    tag = "List",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("clan_id" = Uuid, Path, description = "The clan to lookup the data for"),
    ),
    responses(
        (status = 200, body = ClanProfileResolved),
        (status = 404, description = "Clan not found", body = ErrorResponse)
    )
)]
#[get("/{clan_id}", wrap = "CacheController::public_with_max_age(900)")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<ClanPath>,
) -> Result<HttpResponse, ApiError> {
    let profile = web::block(move || {
        ClanProfileResolved::find(&mut db.connection()?, path.list, path.clan_id)
    })
    .await??;
    Ok(HttpResponse::Ok().json(profile))
}

#[utoipa::path(
    get,
    summary = "Clan level records",
    description = "Get all clan victors for a specific level",
    tag = "List",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("clan_id" = Uuid, Path, description = "The clan to lookup the records for"),
        ("level_id" = Uuid, Path, description = "The level to lookup the records for")
    ),
    responses(
        (status = 200, body = [LevelResolvedRecordExtended]),
    )
)]
#[get(
    "/{clan_id}/levels/{level_id}/records",
    wrap = "CacheController::public_with_max_age(900)"
)]
async fn level_records(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<ClanLevelPath>,
) -> Result<HttpResponse, ApiError> {
    let records = web::block(move || {
        ClanProfileResolved::find_records_for_level(
            &mut db.connection()?,
            path.list,
            path.clan_id,
            path.level_id,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(records))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(ClanProfileResolved, LevelResolvedRecordExtended)),
    paths(find, level_records)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/clan").service(find).service(level_records));
}
