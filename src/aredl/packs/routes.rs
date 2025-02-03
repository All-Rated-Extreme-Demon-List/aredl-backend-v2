use std::sync::Arc;
use actix_web::{delete, HttpResponse, patch, post, web};
use uuid::Uuid;
use utoipa::OpenApi;
use crate::aredl::packs::{Pack, PackCreate, PackUpdate, levels};
use crate::auth::{UserAuth, Permission};
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[utoipa::path(
    post,
    summary = "[Staff]Add pack",
    description = "Creates a new pack",
    tag = "AREDL - Packs",
    request_body = PackCreate,
    responses(
        (status = 200, body = Pack)
    ),
    security(
        ("access_token" = ["PackModify"]),
        ("api_key" = ["PackModify"]),
    ),
)]
#[post("", wrap="UserAuth::require(Permission::PackModify)")]
async fn create(db: web::Data<Arc<DbAppState>>, pack: web::Json<PackCreate>) -> Result<HttpResponse, ApiError> {
    let pack = web::block(
        || Pack::create(db, pack.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(pack))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit pack",
    description = "Edit a pack information",
    tag = "AREDL - Packs",
    params(
        ("id" = Uuid, description = "Internal pack UUID")
    ),
    request_body = PackUpdate,
    responses(
        (status = 200, body = Pack)
    ),
    security(
        ("access_token" = ["PackModify"]),
        ("api_key" = ["PackModify"]),
    ),
)]
#[patch("/{id}", wrap="UserAuth::require(Permission::PackModify)")]
async fn update(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, pack: web::Json<PackUpdate>) -> Result<HttpResponse, ApiError> {
    let pack = web::block(
        || Pack::update(db, id.into_inner(), pack.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(pack))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Remove pack",
    description = "Delete an existing pack",
    tag = "AREDL - Packs",
    params(
        ("id" = Uuid, description = "Internal pack UUID")
    ),
    responses(
        (status = 200, body = Pack)
    ),
    security(
        ("access_token" = ["PackModify"]),
        ("api_key" = ["PackModify"]),
    ),
)]
#[delete("/{id}", wrap="UserAuth::require(Permission::PackModify)")]
async fn delete(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>) -> Result<HttpResponse, ApiError> {
    let pack = web::block(
        || Pack::delete(db, id.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(pack))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL - Packs", description = "Internal endpoints to manage AREDL packs. To fetch packs data, refer to [AREDL - Pack Tiers](#tag/AREDL-Pack-Tiers)")
    ),
    nest(
        (path = "/{pack_id}/levels", api = levels::ApiDoc)
    ),
    components(
        schemas(
            Pack,
            PackCreate,
            PackUpdate,
        )
    ),
    paths(
        create,
        update,
        delete
    ),
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("packs")
            .service(create)
            .service(update)
            .service(delete)
            .configure(levels::init_routes)
    );
}