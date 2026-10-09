use crate::cache_control::CacheController;
use crate::error_handler::{ApiError, ErrorResponse};
use crate::list::profile::ProfileResolved;
use crate::list::List;
use crate::{
    app_data::db::DbAppState,
    auth::{Authenticated, UserAuth},
};
use actix_web::{get, web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;
use utoipa::OpenApi;

#[derive(Deserialize)]
struct ProfilePath {
    list: List,
    id: String,
}

#[utoipa::path(
    get,
    summary = "[AuthPublic]Profile",
    description = "Get a user profile for the selected list",
    tag = "List",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("id" = String, Path, description = "The internal UUID, username or discord ID of the user to lookup the profile for")
    ),
    responses(
        (status = 200, body = ProfileResolved),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security((), ("bearer_token" = [])),
)]
#[get(
    "/{id}",
    wrap = "UserAuth::load()",
    wrap = "CacheController::private_with_max_age(300)"
)]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<ProfilePath>,
    authenticated: Option<Authenticated>,
) -> Result<HttpResponse, ApiError> {
    let profile = web::block(move || {
        ProfileResolved::from_str(&mut db.connection()?, path.list, &path.id, authenticated)
    })
    .await??;
    Ok(HttpResponse::Ok().json(profile))
}

#[derive(OpenApi)]
#[openapi(components(schemas(ProfileResolved)), paths(find))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/profile").service(find));
}
