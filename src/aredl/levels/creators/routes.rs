use actix_web::{delete, get, HttpResponse, patch, post, web};
use uuid::Uuid;
use crate::aredl::levels::creators::Creator;
use crate::error_handler::ApiError;

#[get("")]
async fn find_all(level_id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let creators = web::block(
        || Creator::find_all(level_id.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(creators))
}

#[post("")]
async fn set(level_id: web::Path<Uuid>, creators: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let creators = web::block(
        || Creator::set_all(level_id.into_inner(), creators.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(creators))
}

#[patch("")]
async fn add(level_id: web::Path<Uuid>, creators: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let creators = web::block(
        || Creator::add_all(level_id.into_inner(), creators.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(creators))
}

#[delete("")]
async fn delete(level_id: web::Path<Uuid>, creators: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let creators = web::block(
        || Creator::delete_all(level_id.into_inner(), creators.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(creators))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{level_id}/creators")
            .service(find_all)
            .service(add)
            .service(set)
            .service(delete)
    );
}