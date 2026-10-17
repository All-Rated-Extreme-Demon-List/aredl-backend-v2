use crate::app_data::db::DbAppState;
use crate::cache_control::CacheController;
use crate::error_handler::ApiError;
use crate::roles::RoleResolved;
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "Get important users",
    description = "Get the list of important users by role (List staff and AREDL+)",
    tag = "Users",
    responses(
        (status = 200, body = [RoleResolved]),
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(3600)")]
async fn list(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let roles = db.run(RoleResolved::find_all_public).await?;
    Ok(HttpResponse::Ok().json(roles))
}

#[derive(OpenApi)]
#[openapi(components(schemas(RoleResolved)), paths(list))]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/names").service(list));
}
