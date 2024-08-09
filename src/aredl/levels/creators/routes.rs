use std::sync::Arc;
use actix_web::{delete, get, HttpResponse, patch, post, web};
use uuid::Uuid;
use crate::aredl::levels::creators::Creator;
use crate::aredl::levels::id_resolver::resolve_level_id;
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[get("")]
async fn find_all(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let creators = web::block(
        move || Creator::find_all(db, level_id)
    ).await??;
    Ok(HttpResponse::Ok().json(creators))
}

#[post("")]
async fn set(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>, creators: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let creators = web::block(
        move || Creator::set_all(db, level_id, creators.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(creators))
}

#[patch("")]
async fn add(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>, creators: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let creators = web::block(
        move || Creator::add_all(db, level_id, creators.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(creators))
}

#[delete("")]
async fn delete(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>, creators: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let creators = web::block(
        move || Creator::delete_all(db, level_id, creators.into_inner())
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