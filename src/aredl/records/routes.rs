use std::sync::Arc;
use actix_web::{delete, HttpResponse, patch, post, web};
use uuid::Uuid;
use utoipa::OpenApi;
use crate::auth::{UserAuth, Permission};
use crate::aredl::records::{Record, RecordResolved};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::aredl::records::model::{RecordInsert, RecordUpdate};

#[utoipa::path(
    post,
    summary = "[Staff]Create record",
    description = "Create a new record",
    tag = "AREDL - Records",
    request_body = RecordInsert,
    responses(
        (status = 200, body = Record)
    ),
    security(
        ("access_token" = ["RecordModify"]),
        ("api_key" = ["RecordModify"]),
    )
)]
#[post("", wrap="UserAuth::require(Permission::RecordModify)")]
async fn create(db: web::Data<Arc<DbAppState>>, record: web::Json<RecordInsert>) -> Result<HttpResponse, ApiError> {
    let record = web::block(
        move || Record::create(db, record.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(record))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit record",
    description = "Edit a specific record",
    tag = "AREDL - Records",
    request_body = RecordUpdate,
    params(
        ("id" = Uuid, description = "Internal record UUID")
    ),
    responses(
        (status = 200, body = Record)
    ),
    security(
        ("access_token" = ["RecordModify"]),
        ("api_key" = ["RecordModify"]),
    )
)]
#[patch("/{id}", wrap="UserAuth::require(Permission::RecordModify)")]
async fn update(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, record: web::Json<RecordUpdate>) -> Result<HttpResponse, ApiError> {
	let id = id.into_inner();
    let record = web::block(
        move || Record::update(db, id, record.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(record))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete record",
    description = "Remove a specific record from this level",
    tag = "AREDL - Records",
    params(
        ("id" = Uuid, description = "Internal record UUID")
    ),
    responses(
        (status = 200)
    ),
    security(
        ("access_token" = ["RecordModify"]),
        ("api_key" = ["RecordModify"]),
    )
)]
#[delete("/{id}", wrap="UserAuth::require(Permission::RecordModify)")]
async fn delete(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
	let id = id.into_inner();
    let record = web::block(
        move || Record::delete(db, id)
    ).await??;
    Ok(HttpResponse::Ok().json(record))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL - Records", description = "Endpoints for fetching and managing records")
    ),
    components(
        schemas(
            Record,
            RecordUpdate,
            RecordResolved,
            RecordUpdate,
        )
    ),
    paths(
        create,
        update,
        delete
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/records")
            .service(create)
            .service(update)
            .service(delete)
    );
}