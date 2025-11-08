use crate::cache_control::CacheController;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::users::names::RoleResolved;
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "Get important users",
    description = "Get the list of important users by role (List staff and AREDL+)",
    tag = "Users",
    responses(
        (status = 200, body = RoleResolved)
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(3600)")]
async fn list(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let roles = web::block(move || RoleResolved::find_all(&mut db.connection()?)).await??;
    Ok(HttpResponse::Ok().json(roles))
}

#[derive(OpenApi)]
#[openapi(components(schemas(RoleResolved)), paths(list))]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/names").service(list));
}
