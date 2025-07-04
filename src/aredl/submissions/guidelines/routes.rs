use actix_web::{post, web, HttpResponse, get};
use std::sync::Arc;
use crate::{
    aredl::submissions::guidelines::*,
    auth::{Authenticated, Permission, UserAuth}, 
    db::DbAppState, 
    error_handler::ApiError,
};
use utoipa::OpenApi;
use tracing_actix_web::RootSpan;

#[utoipa::path(
    post,
    summary = "[Staff]Update guidelines",
    description = "Update the submission guidelines shown on the site",
    tag = "AREDL - Submissions",
    responses(
        (status = 200, body = GuidelineUpdateFull)
    ),
    security(
        ("access_token" = []),
    ),
)]
#[post("", wrap="UserAuth::require(Permission::ShiftManage)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    body: web::Json<GuidelineUpdateBody>,
    authenticated: Authenticated,
    root_span: RootSpan
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&body));
    let guidelines = web::block(
        move || GuidelineUpdate::update(&mut db.connection()?, body.into_inner().guidelines, authenticated.user_id)
    ).await??;
    Ok(HttpResponse::Ok().json(guidelines))
}

#[utoipa::path(
    get,
    summary = "Get guidelines",
    description = "Returns the markdown text for the submission guidelines, as well as the moderator who last updated them",
    tag = "AREDL - Submissions",
    responses(
        (status = 200, body = GuidelineUpdateFull)
    ),
    security(
        ("access_token" = []),
    ),
)]
#[get("")]
async fn get_guidelines(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let guidelines = web::block(
        move || GuidelineUpdate::latest(&mut db.connection()?)
    ).await??;
    Ok(HttpResponse::Ok().json(guidelines))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        GuidelineUpdateFull
    )),
    paths(update, get_guidelines)
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/guidelines")
            .service(update)
            .service(get_guidelines)
    );
}
