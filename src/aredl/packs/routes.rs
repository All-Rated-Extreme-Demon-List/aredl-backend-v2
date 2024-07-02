use actix_web::{delete, HttpResponse, patch, post, web};
use uuid::Uuid;
use crate::aredl::packs::{Pack, PackCreate, PackUpdate};
use crate::auth::{UserAuth, Permission};
use crate::error_handler::ApiError;

#[post("", wrap="UserAuth::require(Permission::PackModify)")]
async fn create(pack: web::Json<PackCreate>) -> Result<HttpResponse, ApiError> {
    let pack = web::block(
        || Pack::create(pack.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(pack))
}

#[patch("/{id}", wrap="UserAuth::require(Permission::PackModify)")]
async fn update(id: web::Path<Uuid>, pack: web::Json<PackUpdate>) -> Result<HttpResponse, ApiError> {
    let pack = web::block(
        || Pack::update(id.into_inner(), pack.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(pack))
}

#[delete("/{id}", wrap="UserAuth::require(Permission::PackModify)")]
async fn delete(id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let pack = web::block(
        || Pack::delete(id.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(pack))
}


pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("packs")
            .service(create)
            .service(update)
            .service(delete)
    );
}