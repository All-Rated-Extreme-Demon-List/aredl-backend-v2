use actix_web::{post, web, HttpResponse, get};
use std::sync::Arc;
use crate::{
    aredl::submissions::status::SubmissionsEnabled,
    auth::{Authenticated, Permission, UserAuth}, 
    db::DbAppState, 
    error_handler::ApiError,
};
use utoipa::OpenApi;

#[utoipa::path(
    delete,
    summary = "[Auth]Enable submissions",
    description = "Toggle submissions on, allowing users to submit records to the list",
    tag = "AREDL - Submissions",
    responses(
        (status = 200)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
)]
#[post("/enable", wrap="UserAuth::require(Permission::ShiftManage)")]
async fn enable(db: web::Data<Arc<DbAppState>>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    web::block(
        move || SubmissionsEnabled::enable(&mut db.connection()?, authenticated.user_id)
    ).await??;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    delete,
    summary = "[Auth]Disable submissions",
    description = "Toggle submissions off, stopping users from submitting records to the list.",
    tag = "AREDL - Submissions",
    responses(
        (status = 200)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
)]
#[post("/disable", wrap="UserAuth::require(Permission::ShiftManage)")]
async fn disable(db: web::Data<Arc<DbAppState>>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    web::block(
        move || SubmissionsEnabled::disable(&mut db.connection()?, authenticated.user_id)
    ).await??;
    Ok(HttpResponse::Ok().finish())
}

#[utoipa::path(
    delete,
    summary = "[Auth]Get submission status",
    description = "Get the status of submissions",
    tag = "AREDL - Submissions",
    responses(
        (status = 200)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
)]
#[get("/full", wrap="UserAuth::require(Permission::ShiftManage)")]
async fn get_status_full(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let res = web::block(
        move || SubmissionsEnabled::get_status(&mut db.connection()?)
    ).await??;
    return Ok(HttpResponse::Ok().json(res));
}

#[utoipa::path(
    delete,
    summary = "[Auth]Get submission status",
    description = "Get the status of submissions",
    tag = "AREDL - Submissions",
    responses(
        (status = 200)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
)]
#[get("")]
async fn get_status(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let res = web::block(
        move || SubmissionsEnabled::is_enabled(&mut db.connection()?)
    ).await??;
    return Ok(HttpResponse::Ok().json(res));
}

#[utoipa::path(
    delete,
    summary = "[Auth]Get submission status history",
    description = "Get a log of when submissions were enabled or disabled and by whom.",
    tag = "AREDL - Submissions",
    responses(
        (status = 200)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
)]
#[get("/history")]
async fn get_history(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let res = web::block(
        move || SubmissionsEnabled::get_statuses(&mut db.connection()?)
    ).await??;
    return Ok(HttpResponse::Ok().json(res));
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(
        SubmissionsEnabled
    )),
    paths(get_status, get_status_full, enable, disable, get_history)
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/status")
            .service(get_status)
            .service(get_status_full)
            .service(get_history)
            .service(enable)
            .service(disable)
    );
}
