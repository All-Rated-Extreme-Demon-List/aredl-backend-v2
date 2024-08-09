use std::sync::Arc;
use actix_web::{delete, get, HttpResponse, patch, post, web};
use uuid::Uuid;
use crate::aredl::levels::id_resolver::resolve_level_id;
use crate::auth::{UserAuth, Permission};
use crate::aredl::levels::records::{Record, RecordInsert, RecordResolved, RecordUpdate};
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[get("")]
async fn find_all(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let records = web::block(
        move || RecordResolved::find_all(db, level_id)
    ).await??;
    Ok(HttpResponse::Ok().json(records))
}

#[get("/full", wrap="UserAuth::require(Permission::RecordModify)")]
async fn find_all_full(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let records = web::block(
        move || Record::find_all(db, level_id)
    ).await??;
    Ok(HttpResponse::Ok().json(records))
}

#[post("", wrap="UserAuth::require(Permission::RecordModify)")]
async fn create(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>, record: web::Json<RecordInsert>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let record = web::block(
        move || Record::create(db, level_id, record.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(record))
}

#[patch("/{id}", wrap="UserAuth::require(Permission::RecordModify)")]
async fn update(db: web::Data<Arc<DbAppState>>, path: web::Path<(String, Uuid)>, record: web::Json<RecordUpdate>) -> Result<HttpResponse, ApiError> {
    let (level_id, id) = path.into_inner();
    let level_id = resolve_level_id(&db, level_id.as_str())?;
    let record = web::block(
        move || Record::update(db, level_id, id, record.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(record))
}

#[delete("/{id}", wrap="UserAuth::require(Permission::RecordModify)")]
async fn delete(db: web::Data<Arc<DbAppState>>, path: web::Path<(String, Uuid)>) -> Result<HttpResponse, ApiError> {
    let (level_id, id) = path.into_inner();
    let level_id = resolve_level_id(&db, level_id.as_str())?;
    let record = web::block(
        move || Record::delete(db, level_id, id)
    ).await??;
    Ok(HttpResponse::Ok().json(record))
}

#[get("/{id}", wrap="UserAuth::require(Permission::RecordModify)")]
async fn find(db: web::Data<Arc<DbAppState>>, path: web::Path<(String, Uuid)>) -> Result<HttpResponse, ApiError> {
    let (level_id, id) = path.into_inner();
    let level_id = resolve_level_id(&db, level_id.as_str())?;
    let record = web::block(
        move || Record::find(db, level_id, id)
    ).await??;
    Ok(HttpResponse::Ok().json(record))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{level_id}/records")
            .service(find_all)
            .service(find_all_full)
            .service(find)
            .service(create)
            .service(update)
            .service(delete)
    );
}