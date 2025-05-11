use crate::arepl::profile::ProfileResolved;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "Profile",
    description = "Get an user AREDL platformer profile",
    tag = "AREDL (P)",
    params(
        ("id" = Uuid, description = "The user to lookup the profile for")
    ),
    responses(
        (status = 200, body = ProfileResolved)
    ),
)]
#[get("/{id}")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let profile = web::block(move || {
        let mut conn = db.connection()?;
        ProfileResolved::find(&mut conn, id.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(profile))
}

#[derive(OpenApi)]
#[openapi(components(schemas(ProfileResolved)), paths(find))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("profile").service(find));
}
