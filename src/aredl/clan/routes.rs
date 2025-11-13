use crate::aredl::clan::ClanProfileResolved;
use crate::cache_control::CacheController;
use crate::app_data::db::DbAppState;
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

#[derive(OpenApi)]
#[openapi(components(schemas(ClanProfileResolved)), paths(find))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("clan").service(find));
}
