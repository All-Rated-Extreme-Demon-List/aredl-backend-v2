use crate::app_data::db::DbAppState;
use crate::auth::{Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::error_handler::{ApiError, ErrorResponse};
use crate::list::packs::{levels, CompletedPackVictor, Pack, PackCreate, PackUpdate};
use crate::list::List;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;
#[derive(Debug, Deserialize)]
pub(super) struct PackPath {
    pub list: List,
    pub pack_id: Uuid,
}

#[utoipa::path(
    post,
    summary = "[Staff]Add pack",
    description = "Creates a new pack",
    tag = "List - Packs",
    request_body = PackCreate,
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
    ),
    responses(
        (status = 201, body = Pack),
    ),
    security(("bearer_token" = ["PackModify"])),
)]
#[post("", wrap = "UserAuth::require(Permission::PackModify)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    pack: web::Json<PackCreate>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&pack));
    let pack =
        web::block(move || Pack::create(&mut db.connection()?, *list, pack.into_inner())).await??;
    Ok(HttpResponse::Created().json(pack))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit pack",
    description = "Edit a pack information",
    tag = "List - Packs",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("pack_id" = Uuid, Path, description = "Internal pack UUID")
    ),
    request_body = PackUpdate,
    responses(
        (status = 200, body = Pack),
        (status = 404, description = "Pack not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["PackModify"])),
)]
#[patch("/{pack_id}", wrap = "UserAuth::require(Permission::PackModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<PackPath>,
    pack: web::Json<PackUpdate>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&pack));
    let pack = web::block(move || {
        Pack::update(
            &mut db.connection()?,
            path.list,
            path.pack_id,
            pack.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(pack))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Remove pack",
    description = "Delete an existing pack",
    tag = "List - Packs",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("pack_id" = Uuid, Path, description = "Internal pack UUID")
    ),
    responses(
        (status = 200, body = Pack),
        (status = 404, description = "Pack not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["PackModify"])),
)]
#[delete("/{pack_id}", wrap = "UserAuth::require(Permission::PackModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<PackPath>,
) -> Result<HttpResponse, ApiError> {
    let pack =
        web::block(move || Pack::delete(&mut db.connection()?, path.list, path.pack_id)).await??;
    Ok(HttpResponse::Ok().json(pack))
}

#[utoipa::path(
    get,
    summary = "Get Pack Victors",
    description = "Fetch the list of all users who have completed this pack",
    tag = "List - Packs",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("pack_id" = Uuid, Path, description = "Internal pack UUID")
    ),
    responses(
        (status = 200, body = [CompletedPackVictor]),
    ),
)]
#[get(
    "/{pack_id}/victors",
    wrap = "CacheController::public_with_max_age(900)"
)]
async fn get_victors(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<PackPath>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&path));
    let victors =
        web::block(move || Pack::find_victors(&mut db.connection()?, path.list, path.pack_id))
            .await??;
    Ok(HttpResponse::Ok().json(victors))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "List - Packs", description = "Internal endpoints to manage packs. To fetch packs data, refer to [List - Pack Tiers](#tag/List-Pack-Tiers)")
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
        get_victors,
        create,
        update,
        delete
    ),
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/packs")
            .service(create)
            .service(update)
            .service(delete)
            .service(get_victors)
            .configure(levels::init_routes),
    );
}
