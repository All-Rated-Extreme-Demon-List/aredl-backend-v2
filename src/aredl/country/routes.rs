use crate::app_data::db::DbAppState;
use crate::aredl::country::CountryProfileResolved;
use crate::aredl::levels::records::LevelResolvedRecordExtended;
use crate::cache_control::CacheController;
use crate::error_handler::ApiError;
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "Country",
    description = "Get a country's AREDL profile",
    tag = "AREDL",
    params(("id" = i32, description = "The country to lookup the data for")),
    responses((status = 200, body = CountryProfileResolved))
)]
#[get("/{id}", wrap = "CacheController::public_with_max_age(3600)")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let profile =
        web::block(move || CountryProfileResolved::find(&mut db.connection()?, id.into_inner()))
            .await??;
    Ok(HttpResponse::Ok().json(profile))
}

#[utoipa::path(
    get,
    summary = "Country level records",
    description = "Get all country victors for a specific level",
    tag = "AREDL",
    params(
        ("country" = i32, description = "The country to lookup the records for"),
        ("level_id" = Uuid, description = "The level to lookup the records for")
    ),
    responses((status = 200, body = [LevelResolvedRecordExtended]))
)]
#[get(
    "/{country}/levels/{level_id}/records",
    wrap = "CacheController::public_with_max_age(3600)"
)]
async fn level_records(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<(i32, Uuid)>,
) -> Result<HttpResponse, ApiError> {
    let (country, level_id) = path.into_inner();
    let records = web::block(move || {
        CountryProfileResolved::find_records_for_level(&mut db.connection()?, country, level_id)
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
    config.service(web::scope("country").service(find).service(level_records));
}
