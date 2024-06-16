use actix_web::{delete, get, HttpResponse, patch, post, web};
use uuid::Uuid;
use crate::aredl::levels::LevelId;
use crate::auth::{UserAuth, Permission};
use crate::aredl::levels::records::{Record, RecordInsert, RecordResolved, RecordUpdate};
use crate::error_handler::ApiError;

#[get("")]
async fn find_all(level_id: web::Path<LevelId>) -> Result<HttpResponse, ApiError> {
    let records = web::block(
        || RecordResolved::find_all(level_id.into_inner().into())
    ).await??;
    Ok(HttpResponse::Ok().json(records))
}

#[get("/full", wrap="UserAuth::require(Permission::RecordModify)")]
async fn find_all_full(level_id: web::Path<LevelId>) -> Result<HttpResponse, ApiError> {
    let records = web::block(
        || Record::find_all(level_id.into_inner().into())
    ).await??;
    Ok(HttpResponse::Ok().json(records))
}

#[post("", wrap="UserAuth::require(Permission::RecordModify)")]
async fn create(level_id: web::Path<LevelId>, record: web::Json<RecordInsert>) -> Result<HttpResponse, ApiError> {
    let record = web::block(
        || Record::create(level_id.into_inner().into(), record.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(record))
}

#[patch("/{id}", wrap="UserAuth::require(Permission::RecordModify)")]
async fn update(path: web::Path<(LevelId, Uuid)>, record: web::Json<RecordUpdate>) -> Result<HttpResponse, ApiError> {
    let (level_id, id) = path.into_inner();
    let record = web::block(
        move || Record::update(level_id.into(), id, record.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(record))
}

#[delete("/{id}", wrap="UserAuth::require(Permission::RecordModify)")]
async fn delete(path: web::Path<(LevelId, Uuid)>) -> Result<HttpResponse, ApiError> {
    let (level_id, id) = path.into_inner();
    let record = web::block(
        move || Record::delete(level_id.into(), id)
    ).await??;
    Ok(HttpResponse::Ok().json(record))
}

#[get("/{id}", wrap="UserAuth::require(Permission::RecordModify)")]
async fn find(path: web::Path<(LevelId, Uuid)>) -> Result<HttpResponse, ApiError> {
    let (level_id, id) = path.into_inner();
    let record = web::block(
        move || Record::find(level_id.into(), id)
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