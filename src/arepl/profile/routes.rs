use crate::arepl::profile::ProfileResolved;
use crate::cache_control::CacheController;
use crate::app_data::db::DbAppState;
use crate::error_handler::ApiError;
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "Profile",
    description = "Get an user AREDL platformer profile",
    tag = "AREDL (P)",
    params(
        ("id" = String, description = "The user UUID or discord ID to lookup the profile for")
    ),
    responses(
        (status = 200, body = ProfileResolved)
    ),
)]
#[get("/{id}", wrap = "CacheController::private_with_max_age(300)")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let profile =
        web::block(move || ProfileResolved::from_str(&mut db.connection()?, id.as_str())).await??;
    Ok(HttpResponse::Ok().json(profile))
}

#[derive(OpenApi)]
#[openapi(components(schemas(ProfileResolved)), paths(find))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("profile").service(find));
}
