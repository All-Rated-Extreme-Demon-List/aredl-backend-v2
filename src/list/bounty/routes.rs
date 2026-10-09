use crate::app_data::db::DbAppState;
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::error_handler::{ApiError, ErrorResponse};
use crate::list::bounty::{completions, Bounty, BountyPatch, BountyPost, BountyResolved};
use crate::list::List;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[derive(Deserialize)]
pub(super) struct BountyPath {
    pub list: List,
    pub bounty_id: Uuid,
}

#[utoipa::path(
    get,
    summary = "[AuthPublic]Bounty Board",
    description = "Get the list of bounties",
    tag = "List - Bounty Board",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
    ),
    responses(
        (status = 200, body = [BountyResolved]),
    ),
    security((), ("bearer_token" = [])),
)]
#[get(
    "",
    wrap = "UserAuth::load()",
    wrap = "CacheController::auth_public_with_max_age(300)"
)]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    authenticated: Option<Authenticated>,
) -> Result<HttpResponse, ApiError> {
    let result =
        web::block(move || BountyResolved::find_all(&mut db.connection()?, *list, authenticated))
            .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Create Bounty",
    description = "Adds a new bounty for a level on the bounty board",
    tag = "List - Bounty Board",
    request_body = BountyPost,
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
    ),
    responses(
        (status = 201, body = Bounty),
        (status = 400, description = "End date must be after start date, and target submissions must be a positive integer", body = ErrorResponse, examples(
            ("invalid_dates" = (value = json!({"message": "End date must be after start date."}))),
            ("invalid_target" = (value = json!({"message": "Target submissions must be a positive integer."})))
        )),
    ),
    security(("bearer_token" = ["BountyManage"])),
)]
#[post("", wrap = "UserAuth::require(Permission::BountyManage)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    new_bounty: web::Json<BountyPost>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&new_bounty));
    let result =
        web::block(move || Bounty::create(&mut db.connection()?, *list, new_bounty.into_inner()))
            .await??;
    Ok(HttpResponse::Created().json(result))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Update Bounty",
    description = "Updates an existing bounty on the bounty board",
    tag = "List - Bounty Board",
    request_body = BountyPatch,
    responses(
        (status = 200, body = Bounty),
        (status = 400, description = "End date must be after start date, and target submissions must be a positive integer", body = ErrorResponse, examples(
            ("invalid_dates" = (value = json!({"message": "End date must be after start date."}))),
            ("invalid_target" = (value = json!({"message": "Target submissions must be a positive integer."})))
        )),
        (status = 404, description = "Bounty not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["BountyManage"])),
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("bounty_id" = Uuid, Path, description = "Internal bounty UUID"),
    ),
)]
#[patch("/{bounty_id}", wrap = "UserAuth::require(Permission::BountyManage)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<BountyPath>,
    patch: web::Json<BountyPatch>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&patch));
    let result = web::block(move || {
        let conn = &mut db.connection()?;
        Bounty::find_by_id(conn, path.list, path.bounty_id)?.update(
            conn,
            path.list,
            patch.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete Bounty",
    description = "Deletes a bounty from the bounty board",
    tag = "List - Bounty Board",
    responses(
        (status = 204, description = "Bounty deleted successfully"),
        (status = 404, description = "Bounty not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["BountyManage"])),
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("bounty_id" = Uuid, Path, description = "Internal bounty UUID"),
    ),
)]
#[delete("/{bounty_id}", wrap = "UserAuth::require(Permission::BountyManage)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<BountyPath>,
) -> Result<HttpResponse, ApiError> {
    web::block(move || {
        let conn = &mut db.connection()?;
        Bounty::find_by_id(conn, path.list, path.bounty_id)?.delete(conn, path.list)
    })
    .await??;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(BountyResolved)),
    nest(
        (path = "/{bounty_id}/completions", api = completions::ApiDoc)
    ),
    paths(find_all, create, update, delete)
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/bounty-board")
            .configure(completions::init_routes)
            .service(find_all)
            .service(create)
            .service(update)
            .service(delete),
    );
}
