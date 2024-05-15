use actix_web::{get, post, patch, HttpResponse, web};
use uuid::Uuid;
use crate::aredl::levels::{Level, LevelPlace, LevelUpdate};
use crate::error_handler::CustomError;

#[get("/aredl/levels")]
async fn list() -> Result<HttpResponse, CustomError> {
    let levels = web::block(|| Level::find_all()).await.unwrap()?;
    Ok(HttpResponse::Ok().json(levels))
}

#[post("/aredl/levels")]
async fn create(level: web::Json<LevelPlace>) -> Result<HttpResponse, CustomError> {
    let level = Level::create(level.into_inner())?;
    Ok(HttpResponse::Ok().json(level))
}

#[patch("/aredl/levels/{id}")]
async fn update(id: web::Path<Uuid>,level: web::Json<LevelUpdate>) -> Result<HttpResponse, CustomError> {
    let level = Level::update(id.into_inner(), level.into_inner())?;
    Ok(HttpResponse::Ok().json(level))
}

#[get("/aredl/levels/{id}")]
async fn find(id: web::Path<Uuid>) -> Result<HttpResponse, CustomError> {
    let level = Level::find(id.into_inner());
    Ok(HttpResponse::Ok().json(level))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(list);
    config.service(create);
    config.service(update);
}