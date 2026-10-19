use crate::app_data::db::DbAppState;
use crate::audit::AuditLogEntry;
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::error_handler::{ApiError, ErrorResponse};
use crate::list::tags::{Tag, TagCreate, TagUpdate};
use crate::list::List;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use serde::Deserialize;
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[derive(Deserialize)]
struct TagPath {
    list: List,
    id: Uuid,
}

#[utoipa::path(
    get,
    summary = "Get tags",
    description = "Get available tags information",
    tag = "List - Tags",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
    ),
    responses(
        (status = 200, body = [Tag]),
    ),
)]
#[get("", wrap = "CacheController::public_with_max_age(10800)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
) -> Result<HttpResponse, ApiError> {
    let tags = db.run(move |conn| Tag::find_all(conn, *list)).await?;
    Ok(HttpResponse::Ok().json(tags))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add tag",
    description = "Creates a new tag",
    tag = "List - Tags",
    request_body = TagCreate,
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
    ),
    responses(
        (status = 201, body = Tag),
        (status = 409, description = "Tag name already exists for this list", body = ErrorResponse),
        (status = 422, description = "Tag name must not be blank", body = ErrorResponse),
    ),
    security(("bearer_token" = ["TagModify"])),
)]
#[post("", wrap = "UserAuth::require(Permission::TagModify)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    list: web::Path<List>,
    tag: web::Json<TagCreate>,
    root_span: RootSpan,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&tag));
    let tag = db
        .transaction(move |conn| {
            let tag = Tag::create(conn, *list, tag.into_inner())?;
            AuditLogEntry::log_create(conn, Some(authenticated.user_id), &tag)?;
            Ok::<_, ApiError>(tag)
        })
        .await?;
    Ok(HttpResponse::Created().json(tag))
}
#[utoipa::path(
    patch,
    summary = "[Staff]Edit tag",
    description = "Edits tag information",
    tag = "List - Tags",
    request_body = TagUpdate,
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("id" = Uuid, Path, description = "Internal tag UUID")
    ),
    responses(
        (status = 200, body = Tag),
        (status = 404, description = "Tag not found", body = ErrorResponse),
        (status = 409, description = "Tag name already exists in this list", body = ErrorResponse),
        (status = 422, description = "Tag name must not be blank", body = ErrorResponse),
    ),
    security(("bearer_token" = ["TagModify"])),
)]
#[patch("/{id}", wrap = "UserAuth::require(Permission::TagModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<TagPath>,
    tag: web::Json<TagUpdate>,
    root_span: RootSpan,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&tag));
    let tag = db
        .transaction(move |conn| {
            let before = Tag::find_for_update(conn, path.list, path.id)?;
            let after = Tag::update(conn, path.list, path.id, tag.into_inner())?;
            AuditLogEntry::log_update(conn, Some(authenticated.user_id), &before, &after)?;
            Ok::<_, ApiError>(after)
        })
        .await?;
    Ok(HttpResponse::Ok().json(tag))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete tag",
    description = "Removes a tag",
    tag = "List - Tags",
    params(
        ("list" = List, Path, description = "The selected list. (classic / aredl or platformer / arepl)"),
        ("id" = Uuid, Path, description = "Internal tag UUID")
    ),
    responses(
        (status = 200, body = Tag),
        (status = 404, description = "Tag not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["TagModify"])),
)]
#[delete("/{id}", wrap = "UserAuth::require(Permission::TagModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<TagPath>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let tag = db
        .transaction(move |conn| {
            let deleted = Tag::delete(conn, path.list, path.id)?;
            AuditLogEntry::log_delete(conn, Some(authenticated.user_id), &deleted)?;
            Ok::<_, ApiError>(deleted)
        })
        .await?;
    Ok(HttpResponse::Ok().json(tag))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "List - Tags", description = "Endpoints to fetch and manage tags")
    ),
    components(
        schemas(
            Tag,
            TagCreate,
            TagUpdate
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
        web::scope("/tags")
            .service(find_all)
            .service(create)
            .service(update)
            .service(delete),
    );
}
