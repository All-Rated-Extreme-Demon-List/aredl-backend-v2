use crate::aredl::packtiers::PackTierResolved;
use crate::aredl::packtiers::{PackTier, PackTierCreate, PackTierUpdate};
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "[AuthPublic]Get pack tiers",
    description = "Get all pack tiers (and packs) information.",
    tag = "AREDL - Pack Tiers",
    responses(
        (status = 200, body = PackTierResolved)
    ),
    security(
        (),
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[get(
    "",
    wrap = "UserAuth::load()",
    wrap = "CacheController::public_cache()"
)]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Option<Authenticated>,
) -> Result<HttpResponse, ApiError> {
    let tiers =
        web::block(|| PackTierResolved::find_all(db, authenticated.map(|user| user.user_id)))
            .await??;
    Ok(HttpResponse::Ok().json(tiers))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add tier",
    description = "Creates a new tier",
    tag = "AREDL - Pack Tiers",
    request_body = PackTierCreate,
    responses(
        (status = 200, body = PackTier)
    ),
    security(
        ("access_token" = ["PackTierModify"]),
        ("api_key" = ["PackTierModify"]),
    )
)]
#[post("", wrap = "UserAuth::require(Permission::PackTierModify)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    tier: web::Json<PackTierCreate>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&tier));
    let tier = web::block(|| PackTier::create(db, tier.into_inner())).await??;
    Ok(HttpResponse::Ok().json(tier))
}
#[utoipa::path(
    patch,
    summary = "[Staff]Edit tier",
    description = "Edits a tier base information",
    tag = "AREDL - Pack Tiers",
    request_body = PackTierUpdate,
    params(
        ("id", description = "Internal pack tier UUID")
    ),
    responses(
        (status = 200, body = PackTier)
    ),
    security(
        ("access_token" = ["PackTierModify"]),
        ("api_key" = ["PackTierModify"]),
    )
)]
#[patch("/{id}", wrap = "UserAuth::require(Permission::PackTierModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    tier: web::Json<PackTierUpdate>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&tier));
    let tier = web::block(|| PackTier::update(db, id.into_inner(), tier.into_inner())).await??;
    Ok(HttpResponse::Ok().json(tier))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete tier",
    description = "Removes a packs tier. Does not delete the packs assigned to it",
    tag = "AREDL - Pack Tiers",
    params(
        ("id", description = "Internal pack tier UUID")
    ),
    responses(
        (status = 200, body = PackTier)
    ),
    security(
        ("access_token" = ["PackTierModify"]),
        ("api_key" = ["PackTierModify"]),
    )
)]
#[delete("/{id}", wrap = "UserAuth::require(Permission::PackTierModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let tier = web::block(|| PackTier::delete(db, id.into_inner())).await??;
    Ok(HttpResponse::Ok().json(tier))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL - Pack Tiers", description = "Endpoints to fetch and manage AREDL pack tiers")
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
        web::scope("pack-tiers")
            .service(find_all)
            .service(create)
            .service(update)
            .service(delete),
    );
}
