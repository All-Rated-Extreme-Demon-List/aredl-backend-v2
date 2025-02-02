use std::sync::Arc;
use actix_web::{delete, get, HttpResponse, patch, post, web};
use uuid::Uuid;
use utoipa::OpenApi;
use crate::aredl::levels::id_resolver::resolve_level_id;
use crate::auth::{UserAuth, Permission};
use crate::aredl::levels::records::{Record, RecordInsert, RecordResolved, RecordUpdate};
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[utoipa::path(
    get,
    summary = "List records",
    description = "List all of this levels records",
    tag = "AREDL - Levels (Records)",
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, body = [RecordResolved])
    ),
)]
#[get("")]
async fn find_all(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let records = web::block(
        move || RecordResolved::find_all(db, level_id)
    ).await??;
    Ok(HttpResponse::Ok().json(records))
}

#[utoipa::path(
    get,
    summary = "[Staff]List full records",
    description = "List all of this levels records, resolved with full information",
    tag = "AREDL - Levels (Records)",
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, body = [Record])
    ),
    security(
        ("access_token" = ["RecordModify"]),
        ("api_key" = ["RecordModify"]),
    )
)]
#[get("/full", wrap="UserAuth::require(Permission::RecordModify)")]
async fn find_all_full(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let records = web::block(
        move || Record::find_all(db, level_id)
    ).await??;
    Ok(HttpResponse::Ok().json(records))
}

#[utoipa::path(
    post,
    summary = "[Staff]Create record",
    description = "Create a new record for this level",
    tag = "AREDL - Levels (Records)",
    request_body = RecordInsert,
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)")
    ),
    responses(
        (status = 200, body = Record)
    ),
    security(
        ("access_token" = ["RecordModify"]),
        ("api_key" = ["RecordModify"]),
    )
)]
#[post("", wrap="UserAuth::require(Permission::RecordModify)")]
async fn create(db: web::Data<Arc<DbAppState>>, level_id: web::Path<String>, record: web::Json<RecordInsert>) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let record = web::block(
        move || Record::create(db, level_id, record.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(record))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit record",
    description = "Edit a specific record of this level",
    tag = "AREDL - Levels (Records)",
    request_body = RecordUpdate,
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)"),
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
async fn update(db: web::Data<Arc<DbAppState>>, path: web::Path<(String, Uuid)>, record: web::Json<RecordUpdate>) -> Result<HttpResponse, ApiError> {
    let (level_id, id) = path.into_inner();
    let level_id = resolve_level_id(&db, level_id.as_str())?;
    let record = web::block(
        move || Record::update(db, level_id, id, record.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(record))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete record",
    description = "Remove a specific record from this level",
    tag = "AREDL - Levels (Records)",
    params(
        ("level_id" = String, description = "Level ID (Can be internal UUID, or GD ID. For the latter, add a _2p suffix to target the 2p version)"),
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

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL - Levels (Records)", description = "Endpoints for fetching and managing records of a specific level")
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
        find_all,
        find_all_full,
        create,
        update,
        delete
    )
)]
pub struct ApiDoc;

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