use crate::{
    audit::{AuditAction, AuditEntityType, AuditLogEntry, AuditLogPage, AuditLogQueryOpts},
    auth::{Permission, UserAuth},
    db::DbAppState,
    error_handler::ApiError,
    page_helper::{PageQuery, Paginated},
};
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "[Staff]View audit log",
    description = "Get a possibly filtered list of actions done on the website.",
    tag = "Audit Log",
    responses(
        (status = 200, body = Paginated<AuditLogPage>),
    ),
    params(
        PageQuery<50>,
        ("actor_filter" = Option<Uuid>, Query, description = "Filter by which user is performing the action"),
        ("entity_type_filter" = Option<AuditEntityType>, Query, description = "Filter by the type of entity affected by the action"),
        ("entity_filter" = Option<Uuid>, Query, description = "Filter by the ID of the entity affected by the action"),
        ("action_filter" = Option<AuditAction>, Query, description = "Filter by the type of action performed"),
),
    security(("bearer_token" = ["ViewAuditLog"])),
)]
#[get("", wrap = "UserAuth::require(Permission::ViewAuditLog)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<50>>,
    options: web::Query<AuditLogQueryOpts>,
) -> Result<HttpResponse, ApiError> {
    let logs = web::block(move || {
        AuditLogEntry::list_all(
            &mut db.connection()?,
            &options.into_inner(),
            page_query.into_inner(),
        )
    })
    .await??;

    Ok(HttpResponse::Ok().json(logs))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Audit Log", description = "Endpoints for viewing audit logs")
    ),
    paths(
        find_all,
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/audit-log").service(find_all));
}
