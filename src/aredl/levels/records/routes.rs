use crate::aredl::levels::id_resolver::resolve_level_id;
use crate::aredl::records::Record;
use crate::aredl::records::RecordResolved;
use crate::auth::{Permission, UserAuth};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

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
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    level_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let records = web::block(move || RecordResolved::find_all(db, level_id)).await??;
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
#[get("/full", wrap = "UserAuth::require(Permission::RecordModify)")]
async fn find_all_full(
    db: web::Data<Arc<DbAppState>>,
    level_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let level_id = resolve_level_id(&db, level_id.into_inner().as_str())?;
    let records = web::block(move || Record::find_all(db, level_id)).await??;
    Ok(HttpResponse::Ok().json(records))
}

#[get("/{id}", wrap = "UserAuth::require(Permission::RecordModify)")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<(String, Uuid)>,
) -> Result<HttpResponse, ApiError> {
    let (level_id, id) = path.into_inner();
    let level_id = resolve_level_id(&db, level_id.as_str())?;
    let record = web::block(move || Record::find(db, level_id, id)).await??;
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
            RecordResolved,
        )
    ),
    paths(
        find_all,
        find_all_full,
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{level_id}/records")
            .service(find_all)
            .service(find_all_full)
            .service(find),
    );
}
