use actix_web::{get, post, patch, HttpResponse, web};
use uuid::Uuid;
use crate::aredl::levels::{history, Level, LevelPlace, LevelUpdate};
use crate::error_handler::ApiError;

#[get("/aredl/levels")]
async fn list() -> Result<HttpResponse, ApiError> {
    let levels = web::block(|| Level::find_all()).await??;
    Ok(HttpResponse::Ok().json(levels))
}

#[post("/aredl/levels")]
async fn create(level: web::Json<LevelPlace>) -> Result<HttpResponse, ApiError> {
    let level = web::block(|| Level::create(level.into_inner())).await??;
    Ok(HttpResponse::Ok().json(level))
}

#[patch("/aredl/levels/{id}")]
async fn update(id: web::Path<Uuid>,level: web::Json<LevelUpdate>) -> Result<HttpResponse, ApiError> {
    let level = web::block(|| Level::update(id.into_inner(), level.into_inner())).await??;
    Ok(HttpResponse::Ok().json(level))
}

#[get("/aredl/levels/{id}")]
async fn find(id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let level = web::block(|| Level::find(id.into_inner())).await??;
    Ok(HttpResponse::Ok().json(level))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(list);
    config.service(create);
    config.service(update);
    config.service(find);
    history::init_routes(config);
}