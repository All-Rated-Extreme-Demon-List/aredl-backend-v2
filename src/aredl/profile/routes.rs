use std::sync::Arc;
use actix_web::{get, HttpResponse, web};
use uuid::Uuid;
use utoipa::OpenApi;
use crate::aredl::profile::ProfileResolved;
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[utoipa::path(
    get,
    summary = "Profile",
    description = "Get an user AREDL profile",
    tag = "AREDL",
    params(
        ("id" = Uuid, description = "The user to lookup the profile for")
    ),
    responses(
        (status = 200, body = ProfileResolved)
    ),
)]
#[get("/{id}")]
async fn find(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let profile = web::block(move || {
        let mut conn = db.connection()?;
        ProfileResolved::find(&mut conn, id.into_inner())
    }).await??;
    Ok(HttpResponse::Ok().json(profile))
}

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            ProfileResolved
        )
    ),
    paths(
        find
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("profile")
            .service(find)
    );
}