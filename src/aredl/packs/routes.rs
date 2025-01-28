use std::sync::Arc;
use actix_web::{delete, HttpResponse, patch, post, web};
use uuid::Uuid;
use crate::aredl::packs::{Pack, PackCreate, PackUpdate, levels};
use crate::auth::{UserAuth, Permission};
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[post("", wrap="UserAuth::require(Permission::PackModify)")]
async fn create(db: web::Data<Arc<DbAppState>>, pack: web::Json<PackCreate>) -> Result<HttpResponse, ApiError> {
    let pack = web::block(
        || Pack::create(db, pack.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(pack))
}

#[patch("/{id}", wrap="UserAuth::require(Permission::PackModify)")]
async fn update(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, pack: web::Json<PackUpdate>) -> Result<HttpResponse, ApiError> {
    let pack = web::block(
        || Pack::update(db, id.into_inner(), pack.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(pack))
}

#[delete("/{id}", wrap="UserAuth::require(Permission::PackModify)")]
async fn delete(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let pack = web::block(
        || Pack::delete(db, id.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(pack))
}


pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("packs")
            .service(create)
            .service(update)
            .service(delete)
            .configure(levels::init_routes)
    );
}