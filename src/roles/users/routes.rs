use crate::app_data::db::DbAppState;
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::error_handler::ApiError;
use crate::users::BaseUser;
use actix_web::{delete, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    post,
    summary = "[Staff]Set role users",
    description = "Set all the users of a role",
    tag = "Roles",
    params(
        ("id" = i32, description = "Internal role ID")
    ),
    request_body = [Uuid],
    responses(
        (status = 200, body = [BaseUser])
    ),
    security(
        ("access_token" = ["RoleManage"]),
        ("api_key" = ["RoleManage"]),
    ),

)]
#[post("", wrap = "UserAuth::require(Permission::RoleManage)")]
async fn set(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<i32>,
    authenticated: Authenticated,
    users: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&users));
    let users = web::block(move || {
        BaseUser::role_set_all(
            &mut db.connection()?,
            *id,
            authenticated,
            users.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(users))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Add role users",
    description = "Assign this role to the given users",
    tag = "Roles",
    params(
        ("id" = i32, description = "Internal role ID")
    ),
    request_body = [Uuid],
    responses(
        (status = 200, body = [BaseUser])
    ),
    security(
        ("access_token" = ["RoleManage"]),
        ("api_key" = ["RoleManage"]),
    ),

)]
#[patch("", wrap = "UserAuth::require(Permission::RoleManage)")]
async fn add(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<i32>,
    authenticated: Authenticated,
    users: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&users));
    let users = web::block(move || {
        BaseUser::role_add_all(
            &mut db.connection()?,
            *id,
            authenticated,
            users.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(users))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete role users",
    description = "Removes this role from the given users",
    tag = "Roles",
    params(
        ("id" = i32, description = "Internal role ID")
    ),
    request_body = [Uuid],
    responses(
        (status = 200, body = [BaseUser])
    ),
    security(
        ("access_token" = ["RoleManage"]),
        ("api_key" = ["RoleManage"]),
    ),

)]
#[delete("", wrap = "UserAuth::require(Permission::RoleManage)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<i32>,
    authenticated: Authenticated,

    users: web::Json<Vec<Uuid>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&users));
    let users = web::block(move || {
        BaseUser::role_delete_all(
            &mut db.connection()?,
            *id,
            authenticated,
            users.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(users))
}

#[derive(OpenApi)]
#[openapi(paths(add, set, delete))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{id}/users")
            .service(add)
            .service(set)
            .service(delete),
    );
}
