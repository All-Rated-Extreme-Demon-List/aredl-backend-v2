use crate::arepl::packs::{levels, Pack, PackCreate, PackUpdate};
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
    summary = "[Staff]Add pack",
    description = "Creates a new pack",
    tag = "AREDL (P) - Packs",
    request_body = PackCreate,
    responses(
        (status = 200, body = Pack)
    ),
    security(
        ("access_token" = ["PackModify"]),
        ("api_key" = ["PackModify"]),
    ),
)]
#[post("", wrap = "UserAuth::require(Permission::PackModify)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    pack: web::Json<PackCreate>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&pack));
    let pack = web::block(move || Pack::create(&mut db.connection()?, pack.into_inner())).await??;
    Ok(HttpResponse::Ok().json(pack))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit pack",
    description = "Edit a pack information",
    tag = "AREDL (P) - Packs",
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
#[patch("/{id}", wrap = "UserAuth::require(Permission::PackModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    pack: web::Json<PackUpdate>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&pack));
    let pack =
        web::block(move || Pack::update(&mut db.connection()?, id.into_inner(), pack.into_inner()))
            .await??;
    Ok(HttpResponse::Ok().json(pack))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Remove pack",
    description = "Delete an existing pack",
    tag = "AREDL (P) - Packs",
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
#[delete("/{id}", wrap = "UserAuth::require(Permission::PackModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let pack = web::block(move || Pack::delete(&mut db.connection()?, id.into_inner())).await??;
    Ok(HttpResponse::Ok().json(pack))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL (P) - Packs", description = "Internal endpoints to manage AREDL platformer packs. To fetch packs data, refer to [AREDL (P) - Pack Tiers](#tag/AREDL-P-Pack-Tiers)")
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
            .configure(levels::init_routes),
    );
}
