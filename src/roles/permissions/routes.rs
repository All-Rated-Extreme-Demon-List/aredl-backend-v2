use crate::app_data::db::DbAppState;
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::error_handler::ApiError;
use crate::roles::Role;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "[Staff]List role permissions",
    description = "Get the direct permissions assigned to a role",
    tag = "Roles",
    params(
        ("id" = i32, description = "Internal role ID")
    ),
    responses(
        (status = 200, body = [String])
    ),
    security(
        ("access_token" = ["RoleModify"]),
        ("api_key" = ["RoleModify"]),
    ),
)]
#[get("", wrap = "UserAuth::require(Permission::RoleModify)")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<i32>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let permissions =
        web::block(move || Role::permission_find_all(&mut db.connection()?, *id, &authenticated))
            .await??;
    Ok(HttpResponse::Ok().json(permissions))
}

#[utoipa::path(
    get,
    path = "/resolved",
    summary = "[Staff]List resolved role permissions",
    description = "Get the direct and inherited permissions assigned to a role",
    tag = "Roles",
    params(
        ("id" = i32, description = "Internal role ID")
    ),
    responses(
        (status = 200, body = [String])
    ),
    security(
        ("access_token" = ["RoleModify"]),
        ("api_key" = ["RoleModify"]),
    ),
)]
#[get("/resolved", wrap = "UserAuth::require(Permission::RoleModify)")]
async fn find_all_resolved(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<i32>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let permissions = web::block(move || {
        Role::permission_find_all_resolved(&mut db.connection()?, *id, &authenticated)
    })
    .await??;
    Ok(HttpResponse::Ok().json(permissions))
}

#[utoipa::path(
    post,
    summary = "[Staff]Set role permissions",
    description = "Set all direct permissions assigned to a role",
    tag = "Roles",
    params(
        ("id" = i32, description = "Internal role ID")
    ),
    request_body = [String],
    responses(
        (status = 200, body = [String])
    ),
    security(
        ("access_token" = ["RoleModify"]),
        ("api_key" = ["RoleModify"]),
    ),
)]
#[post("", wrap = "UserAuth::require(Permission::RoleModify)")]
async fn set(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<i32>,
    authenticated: Authenticated,
    permissions: web::Json<Vec<String>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&permissions));
    let permissions = web::block(move || {
        Role::permission_set_all(
            &mut db.connection()?,
            *id,
            authenticated,
            permissions.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(permissions))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Add role permissions",
    description = "Assign direct permissions to a role",
    tag = "Roles",
    params(
        ("id" = i32, description = "Internal role ID")
    ),
    request_body = [String],
    responses(
        (status = 200, body = [String])
    ),
    security(
        ("access_token" = ["RoleModify"]),
        ("api_key" = ["RoleModify"]),
    ),
)]
#[patch("", wrap = "UserAuth::require(Permission::RoleModify)")]
async fn add(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<i32>,
    authenticated: Authenticated,
    permissions: web::Json<Vec<String>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&permissions));
    let permissions = web::block(move || {
        Role::permission_add_all(
            &mut db.connection()?,
            *id,
            authenticated,
            permissions.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(permissions))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete role permissions",
    description = "Removes direct permissions from a role",
    tag = "Roles",
    params(
        ("id" = i32, description = "Internal role ID")
    ),
    request_body = [String],
    responses(
        (status = 200, body = [String])
    ),
    security(
        ("access_token" = ["RoleModify"]),
        ("api_key" = ["RoleModify"]),
    ),
)]
#[delete("", wrap = "UserAuth::require(Permission::RoleModify)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<i32>,
    authenticated: Authenticated,
    permissions: web::Json<Vec<String>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&permissions));
    let permissions = web::block(move || {
        Role::permission_delete_all(
            &mut db.connection()?,
            *id,
            authenticated,
            permissions.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(permissions))
}

#[derive(OpenApi)]
#[openapi(paths(add, set, delete, find_all, find_all_resolved))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{id}/permissions")
            .service(find_all)
            .service(find_all_resolved)
            .service(add)
            .service(set)
            .service(delete),
    );
}
