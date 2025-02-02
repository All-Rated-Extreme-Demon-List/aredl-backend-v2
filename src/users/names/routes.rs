use std::sync::Arc;
use actix_web::{get, HttpResponse, web};
use utoipa::OpenApi;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::users::names::RoleResolved;

#[utoipa::path(
    get,
    summary = "Get important users",
    description = "Get the list of important users by role (List staff and AREDL+)",
    tag = "Users",
    responses(
        (status = 200, body = RoleResolved)
    ),
)]
#[get("")]
async fn list(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let roles = web::block(move || {
        let mut conn = db.connection()?;
        RoleResolved::find_all(&mut conn)
    }).await??;
    Ok(HttpResponse::Ok().json(roles))
}

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            RoleResolved
        )
    ),
    paths(
        list
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/names")
            .service(list)
    );
}