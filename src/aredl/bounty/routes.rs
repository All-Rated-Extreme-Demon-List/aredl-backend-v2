use crate::app_data::db::DbAppState;
use crate::aredl::bounty::{completions, Bounty, BountyPatch, BountyPost, BountyResolved};
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::error_handler::ApiError;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "Bounty Board",
    description = "Get the list of bounties",
    tag = "AREDL - Bounty Board",
    responses(
        (status = 200, body = [Vec<BountyResolved>])
    ),
)]
#[get(
    "",
    wrap = "UserAuth::load()",
    wrap = "CacheController::auth_public_with_max_age(300)"
)]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Option<Authenticated>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || BountyResolved::find_all(&mut db.connection()?, authenticated))
        .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "Create Bounty",
    description = "Adds a new bounty for a level on the bounty board",
    tag = "AREDL - Bounty Board",
    request_body = BountyPost,
    responses(
        (status = 200, body = Bounty)
    ),
)]
#[post("", wrap = "UserAuth::require(Permission::BountyManage)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    new_bounty: web::Json<BountyPost>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&new_bounty));
    let result = web::block(move || Bounty::create(&mut db.connection()?, new_bounty.into_inner()))
        .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    patch,
    summary = "Update Bounty",
    description = "Updates an existing bounty on the bounty board",
    tag = "AREDL - Bounty Board",
    request_body = BountyPatch,
    responses(
        (status = 200, body = Bounty)
    ),
)]
#[patch("/{id}", wrap = "UserAuth::require(Permission::BountyManage)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    patch: web::Json<BountyPatch>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&patch));
    let result = web::block(move || {
        let conn = &mut db.connection()?;
        Bounty::find_by_id(conn, id.into_inner())?.update(conn, patch.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    delete,
    summary = "Delete Bounty",
    description = "Deletes a bounty from the bounty board",
    tag = "AREDL - Bounty Board",
    responses(
        (status = 200, description = "Bounty deleted successfully")
    ),
)]
#[delete("/{id}", wrap = "UserAuth::require(Permission::BountyManage)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    web::block(move || {
        let conn = &mut db.connection()?;
        Bounty::find_by_id(conn, id.into_inner())?.delete(conn)
    })
    .await??;
    Ok(HttpResponse::Ok().finish())
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(BountyResolved)),
    nest(
        (path = "/{bounty_id}/completions", api = completions::ApiDoc)
    ),
    paths(list, create, update, delete)
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/bounty-board")
            .configure(completions::init_routes)
            .service(list)
            .service(create)
            .service(update)
            .service(delete),
    );
}
