use crate::app_data::db::DbAppState;
use crate::cache_control::CacheController;
use crate::error_handler::ApiError;
use crate::list::country::CountryProfileResolved;
use crate::list::levels::records::LevelResolvedRecordExtended;
use crate::list::List;
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

#[derive(Deserialize)]
struct CountryPath {
    list: List,
    country: i32,
}

#[derive(Deserialize)]
struct CountryLevelPath {
    list: List,
    country: i32,
    level_id: Uuid,
}

#[utoipa::path(
    get,
    summary = "Country",
    description = "Get a country's profile for the selected list",
    tag = "List",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("country" = i32, Path, description = "The country to lookup the data for"),
    ),
    responses(
        (status = 200, body = CountryProfileResolved),
    )
)]
#[get("/{country}", wrap = "CacheController::public_with_max_age(3600)")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<CountryPath>,
) -> Result<HttpResponse, ApiError> {
    let profile = web::block(move || {
        CountryProfileResolved::find(&mut db.connection()?, path.list, path.country)
    })
    .await??;
    Ok(HttpResponse::Ok().json(profile))
}

#[utoipa::path(
    get,
    summary = "Country level records",
    description = "Get all country victors for a specific level",
    tag = "List",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("country" = i32, Path, description = "The country to lookup the records for"),
        ("level_id" = Uuid, Path, description = "The level to lookup the records for")
    ),
    responses(
        (status = 200, body = [LevelResolvedRecordExtended]),
    )
)]
#[get(
    "/{country}/levels/{level_id}/records",
    wrap = "CacheController::public_with_max_age(3600)"
)]
async fn level_records(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<CountryLevelPath>,
) -> Result<HttpResponse, ApiError> {
    let records = web::block(move || {
        CountryProfileResolved::find_records_for_level(
            &mut db.connection()?,
            path.list,
            path.country,
            path.level_id,
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(records))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(CountryProfileResolved, LevelResolvedRecordExtended)),
    paths(find, level_records)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/country").service(find).service(level_records));
}
