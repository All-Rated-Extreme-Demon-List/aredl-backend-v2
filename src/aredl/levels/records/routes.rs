use actix_web::{get, HttpResponse, post, web};
use uuid::Uuid;
use crate::auth::{UserAuth, Permission};
use crate::aredl::levels::records::{Record, RecordInsert, RecordResolved, RecordUpdate};
use crate::error_handler::ApiError;

#[get("")]
async fn find_all(level_id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let records = web::block(
        || RecordResolved::find_all(level_id.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(records))
}

#[get("/full", wrap="UserAuth::require(Permission::RecordModify)")]
async fn find_all_full(level_id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let records = web::block(
        || Record::find_all(level_id.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(records))
}

#[post("", wrap="UserAuth::require(Permission::RecordModify)")]
async fn create(level_id: web::Path<Uuid>, record: web::Json<RecordInsert>) -> Result<HttpResponse, ApiError> {
    let record = web::block(
        || Record::create(level_id.into_inner(), record.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(record))
}

#[post("/{id}", wrap="UserAuth::require(Permission::RecordModify)")]
async fn update(level_id: web::Path<Uuid>, id: web::Path<Uuid>, record: web::Json<RecordUpdate>) -> Result<HttpResponse, ApiError> {
    let record = web::block(
        || Record::update(level_id.into_inner(), id.into_inner(), record.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(record))
}

#[get("/{id}", wrap="UserAuth::require(Permission::RecordModify)")]
async fn find(level_id: web::Path<Uuid>, id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let record = web::block(
        || Record::find(level_id.into_inner(), id.into_inner())
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
    );
}