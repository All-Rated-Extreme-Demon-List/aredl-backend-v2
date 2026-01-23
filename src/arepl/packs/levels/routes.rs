use crate::arepl::levels::BaseLevel;
use crate::auth::{Permission, UserAuth};
use crate::app_data::db::DbAppState;
use crate::error_handler::ApiError;
use actix_web::{delete, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    post,
    summary = "[Staff]Set pack levels",
    description = "Set all the levels of a pack",
    tag = "AREDL (P) - Packs",
    params(
        ("pack_id" = Uuid, description = "Internal pack UUID")
    ),
    request_body = [Uuid],
    responses(
        (status = 200, body = [BaseLevel])
    ),
    security(
        ("access_token" = ["PackModify"]),
        ("api_key" = ["PackModify"]),
    ),

)]
#[post("", wrap = "UserAuth::require(Permission::PackModify)")]
async fn set(
    db: web::Data<Arc<DbAppState>>,
    pack_id: web::Path<Uuid>,
    levels: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&levels));
    let levels = web::block(move || {
        BaseLevel::pack_set_all(&mut db.connection()?, *pack_id, levels.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(levels))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Add pack levels",
    description = "Add the given levels to this pack's levels list",
    tag = "AREDL (P) - Packs",
    params(
        ("pack_id" = Uuid, description = "Internal pack UUID")
    ),
    request_body = [Uuid],
    responses(
        (status = 200, body = [BaseLevel])
    ),
    security(
        ("access_token" = ["PackModify"]),
        ("api_key" = ["PackModify"]),
    ),

)]
#[patch("", wrap = "UserAuth::require(Permission::PackModify)")]
async fn add(
    db: web::Data<Arc<DbAppState>>,
    pack_id: web::Path<Uuid>,
    levels: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&levels));
    let levels = web::block(move || {
        BaseLevel::pack_add_all(&mut db.connection()?, *pack_id, levels.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(levels))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete pack levels",
    description = "Removes the given levels from this pack's levels list",
    tag = "AREDL (P) - Packs",
    params(
        ("pack_id" = Uuid, description = "Internal pack UUID")
    ),
    request_body = [Uuid],
    responses(
        (status = 200, body = [BaseLevel])
    ),
    security(
        ("access_token" = ["PackModify"]),
        ("api_key" = ["PackModify"]),
    ),

)]
#[delete("", wrap = "UserAuth::require(Permission::PackModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    pack_id: web::Path<Uuid>,
    levels: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&levels));
    let levels = web::block(move || {
        BaseLevel::pack_delete_all(&mut db.connection()?, *pack_id, levels.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(levels))
}

#[derive(OpenApi)]
#[openapi(paths(add, set, delete))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{pack_id}/levels")
            .service(add)
            .service(set)
            .service(delete),
    );
}
