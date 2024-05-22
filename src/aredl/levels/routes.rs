use actix_web::{get, post, patch, HttpResponse, web};
use uuid::Uuid;
use crate::aredl::levels;
use crate::auth::{Authenticated, UserAuth};
use crate::aredl::levels::{history, Level, LevelPlace, LevelUpdate};
use crate::error_handler::ApiError;

const PERM_MODIFY_LEVELS: &str = "level_modify";

#[get("")]
async fn list() -> Result<HttpResponse, ApiError> {
    let levels = web::block(|| Level::find_all()).await??;
    Ok(HttpResponse::Ok().json(levels))
}

#[post("", wrap="UserAuth::require(PERM_MODIFY_LEVELS)")]
async fn create(level: web::Json<LevelPlace>) -> Result<HttpResponse, ApiError> {
    let level = web::block(|| Level::create(level.into_inner())).await??;
    Ok(HttpResponse::Ok().json(level))
}

#[patch("/{id}", wrap="UserAuth::require(PERM_MODIFY_LEVELS)")]
async fn update(id: web::Path<Uuid>,level: web::Json<LevelUpdate>) -> Result<HttpResponse, ApiError> {
    let level = web::block(|| Level::update(id.into_inner(), level.into_inner())).await??;
    Ok(HttpResponse::Ok().json(level))
}

#[get("/{id}")]
async fn find(id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let level = web::block(|| Level::find(id.into_inner())).await??;
    Ok(HttpResponse::Ok().json(level))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/levels")
            .service(list)
            .service(create)
            .service(update)
            .service(find)
            .configure(history::init_routes)
    );
}