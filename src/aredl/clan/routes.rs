use crate::app_data::db::DbAppState;
use crate::aredl::clan::ClanProfileResolved;
use crate::aredl::levels::records::LevelResolvedRecordExtended;
use crate::cache_control::CacheController;
use crate::error_handler::ApiError;
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "Clan",
    description = "Get a clan's AREDL profile",
    tag = "AREDL",
    params(("id" = Uuid, description = "The clan to lookup the data for")),
    responses((status = 200, body = ClanProfileResolved))
)]
#[get("/{id}", wrap = "CacheController::public_with_max_age(900)")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let profile =
        web::block(move || ClanProfileResolved::find(&mut db.connection()?, id.into_inner()))
            .await??;
    Ok(HttpResponse::Ok().json(profile))
}

#[utoipa::path(
    get,
    summary = "Clan level records",
    description = "Get all clan victors for a specific level",
    tag = "AREDL",
    params(
        ("clan_id" = Uuid, description = "The clan to lookup the records for"),
        ("level_id" = Uuid, description = "The level to lookup the records for")
    ),
    responses((status = 200, body = [LevelResolvedRecordExtended]))
)]
#[get(
    "/{clan_id}/levels/{level_id}/records",
    wrap = "CacheController::public_with_max_age(900)"
)]
async fn level_records(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<(Uuid, Uuid)>,
) -> Result<HttpResponse, ApiError> {
    let (clan_id, level_id) = path.into_inner();
    let records = web::block(move || {
        ClanProfileResolved::find_records_for_level(&mut db.connection()?, clan_id, level_id)
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
    config.service(web::scope("clan").service(find).service(level_records));
}
