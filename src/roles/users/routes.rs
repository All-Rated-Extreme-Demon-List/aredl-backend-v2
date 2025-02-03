use std::sync::Arc;
use actix_web::{delete, HttpResponse, patch, post, web};
use uuid::Uuid;
use utoipa::OpenApi;
use crate::users::BaseUser;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::auth::{UserAuth, Permission};

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
#[post("", wrap="UserAuth::require(Permission::RoleManage)")]
async fn set(db: web::Data<Arc<DbAppState>>, id: web::Path<i32>, users: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let users = web::block(
        move || BaseUser::role_set_all(db, *id, users.into_inner())
    ).await??;
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
#[patch("", wrap="UserAuth::require(Permission::RoleManage)")]
async fn add(db: web::Data<Arc<DbAppState>>, id: web::Path<i32>, users: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let users = web::block(
        move || BaseUser::role_add_all(db, *id, users.into_inner())
    ).await??;
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
#[delete("", wrap="UserAuth::require(Permission::RoleManage)")]
async fn delete(db: web::Data<Arc<DbAppState>>, id: web::Path<i32>, users: web::Json<Vec<Uuid>>) -> Result<HttpResponse, ApiError> {
    let users = web::block(
        move || BaseUser::role_delete_all(db, *id, users.into_inner())
    ).await??;
    Ok(HttpResponse::Ok().json(users))
}

#[derive(OpenApi)]
#[openapi(
    paths(
        add,
        set,
        delete
    ),
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{id}/users")
            .service(add)
            .service(set)
            .service(delete)
    );
}