use std::sync::Arc;
use actix_web::{delete, HttpResponse, patch, post, web};
use uuid::Uuid;
use crate::aredl::packs::levels::Level;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::auth::{UserAuth, Permission};

#[post("", wrap="UserAuth::require(Permission::PackModify)")]
async fn set(db: web::Data<Arc<DbAppState>>, pack_id: web::Path<Uuid>, levels: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let levels = web::block(
        move || Level::set_all(db, *pack_id, levels.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(levels))
}

#[patch("", wrap="UserAuth::require(Permission::PackModify)")]
async fn add(db: web::Data<Arc<DbAppState>>, pack_id: web::Path<Uuid>, levels: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let levels = web::block(
        move || Level::add_all(db, *pack_id, levels.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(levels))
}

#[delete("", wrap="UserAuth::require(Permission::PackModify)")]
async fn delete(db: web::Data<Arc<DbAppState>>, pack_id: web::Path<Uuid>, levels: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let levels = web::block(
        move || Level::delete_all(db, *pack_id, levels.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(levels))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{pack_id}/levels")
            .service(add)
            .service(set)
            .service(delete)
    );
}