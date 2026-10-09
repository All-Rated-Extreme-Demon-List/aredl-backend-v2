use crate::app_data::db::DbAppState;
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::error_handler::{ApiError, ErrorResponse};
use crate::list::packtiers::PackTierResolved;
use crate::list::packtiers::{PackTier, PackTierCreate, PackTierUpdate};
use crate::list::List;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[derive(Deserialize)]
struct PackTierPath {
    list: List,
    id: Uuid,
}

#[utoipa::path(
    get,
    summary = "[AuthPublic]Get pack tiers",
    description = "Get all pack tiers (and packs) information.",
    tag = "List - Pack Tiers",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
    ),
    responses(
        (status = 200, body = [PackTierResolved]),
    ),
    security((), ("bearer_token" = [])),
)]
#[get(
    "",
    wrap = "UserAuth::load()",
    wrap = "CacheController::auth_public_with_max_age(900)"
)]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    authenticated: Option<Authenticated>,
) -> Result<HttpResponse, ApiError> {
    let tiers = web::block(move || {
        PackTierResolved::find_all(
            &mut db.connection()?,
            *list,
            authenticated.map(|user| user.user_id),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(tiers))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add tier",
    description = "Creates a new tier",
    tag = "List - Pack Tiers",
    request_body = PackTierCreate,
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
    ),
    responses(
        (status = 201, body = PackTier),
    ),
    security(("bearer_token" = ["PackTierModify"])),
)]
#[post("", wrap = "UserAuth::require(Permission::PackTierModify)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    tier: web::Json<PackTierCreate>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&tier));
    let tier =
        web::block(move || PackTier::create(&mut db.connection()?, *list, tier.into_inner()))
            .await??;
    Ok(HttpResponse::Created().json(tier))
}
#[utoipa::path(
    patch,
    summary = "[Staff]Edit tier",
    description = "Edits a tier base information",
    tag = "List - Pack Tiers",
    request_body = PackTierUpdate,
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("id" = Uuid, Path, description = "Internal pack tier UUID")
    ),
    responses(
        (status = 200, body = PackTier),
        (status = 404, description = "Pack tier not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["PackTierModify"])),
)]
#[patch("/{id}", wrap = "UserAuth::require(Permission::PackTierModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<PackTierPath>,
    tier: web::Json<PackTierUpdate>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&tier));
    let tier = web::block(move || {
        PackTier::update(&mut db.connection()?, path.list, path.id, tier.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(tier))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete tier",
    description = "Removes a packs tier. Does not delete the packs assigned to it",
    tag = "List - Pack Tiers",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("id" = Uuid, Path, description = "Internal pack tier UUID")
    ),
    responses(
        (status = 200, body = PackTier),
        (status = 404, description = "Pack tier not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["PackTierModify"])),
)]
#[delete("/{id}", wrap = "UserAuth::require(Permission::PackTierModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<PackTierPath>,
) -> Result<HttpResponse, ApiError> {
    let tier =
        web::block(move || PackTier::delete(&mut db.connection()?, path.list, path.id)).await??;
    Ok(HttpResponse::Ok().json(tier))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "List - Pack Tiers", description = "Endpoints to fetch and manage AREDL pack tiers")
    ),
    components(
        schemas(
            PackTier,
            PackTierCreate,
            PackTierResolved,
            PackTierUpdate
        )
    ),
    paths(
        find_all,
        create,
        update,
        delete
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/pack-tiers")
            .service(find_all)
            .service(create)
            .service(update)
            .service(delete),
    );
}
