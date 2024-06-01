use actix_web::{get, post, patch, HttpResponse, web};
use uuid::Uuid;
use crate::auth::{UserAuth, Permission};
use crate::aredl::levels::{history, packs, Level, LevelPlace, LevelUpdate, ResolvedLevel};
use crate::error_handler::ApiError;

#[get("")]
async fn list() -> Result<HttpResponse, ApiError> {
    let levels = web::block(
        || Level::find_all()
    ).await??;
    Ok(HttpResponse::Ok().json(levels))
}

#[post("", wrap="UserAuth::require(Permission::LevelModify)")]
async fn create(level: web::Json<LevelPlace>) -> Result<HttpResponse, ApiError> {
    let level = web::block(
        || Level::create(level.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(level))
}

#[patch("/{id}", wrap="UserAuth::require(Permission::LevelModify)")]
async fn update(id: web::Path<Uuid>,level: web::Json<LevelUpdate>) -> Result<HttpResponse, ApiError> {
    let level = web::block(
        || Level::update(id.into_inner(), level.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(level))
}

#[get("/{id}")]
async fn find(id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let level = web::block(
        || ResolvedLevel::find(id.into_inner())
    ).await??;
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
            .configure(packs::init_routes)
    );
}