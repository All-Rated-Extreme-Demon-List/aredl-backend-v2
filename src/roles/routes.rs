use crate::app_data::db::DbAppState;
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::error_handler::ApiError;
use crate::roles::{users, Role, RoleCreate, RoleUpdate};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;

#[utoipa::path(
	get,
	summary = "List roles",
	description = "Get the list of all roles and their base information",
	tag = "Roles",
	responses(
		(status = 200, body = [Role])
	),
)]
#[get("")]
async fn find_all(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let roles = web::block(move || Role::find_all(&mut db.connection()?)).await??;
    Ok(HttpResponse::Ok().json(roles))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add role",
    description = "Creates a new role",
    tag = "Roles",
    request_body = RoleCreate,
    responses(
        (status = 200, body = Role)
    ),
    security(
        ("access_token" = ["RoleManage"]),
        ("api_key" = ["RoleManage"]),
    ),
)]
#[post("", wrap = "UserAuth::require(Permission::RoleManage)")]
async fn create(
    db: web::Data<Arc<DbAppState>>,
    role: web::Json<RoleCreate>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&role));
    let role =
        web::block(move || Role::create(&mut db.connection()?, authenticated, role.into_inner()))
            .await??;
    Ok(HttpResponse::Ok().json(role))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit role",
    description = "Edit a role information",
    tag = "Roles",
    params(
        ("id" = i32, description = "Internal role ID")
    ),
    request_body = RoleUpdate,
    responses(
        (status = 200, body = Role)
    ),
    security(
        ("access_token" = ["RoleManage"]),
        ("api_key" = ["RoleManage"]),
    ),
)]
#[patch("/{id}", wrap = "UserAuth::require(Permission::RoleManage)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<i32>,
    authenticated: Authenticated,
    role: web::Json<RoleUpdate>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&role));
    let role = web::block(move || {
        Role::update(
            &mut db.connection()?,
            authenticated,
            id.into_inner(),
            role.into_inner(),
        )
    })
    .await??;
    Ok(HttpResponse::Ok().json(role))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Remove role",
    description = "Delete an existing role",
    tag = "Roles",
    params(
        ("id" = i32, description = "Internal role ID")
    ),
    responses(
        (status = 200, body = Role)
    ),
    security(
        ("access_token" = ["RoleManage"]),
        ("api_key" = ["RoleManage"]),
    ),
)]
#[delete("/{id}", wrap = "UserAuth::require(Permission::RoleManage)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
    id: web::Path<i32>,
) -> Result<HttpResponse, ApiError> {
    let role =
        web::block(move || Role::delete(&mut db.connection()?, authenticated, id.into_inner()))
            .await??;
    Ok(HttpResponse::Ok().json(role))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Roles", description = "Internal endpoints to manage staff roles. Only available to developers and owners.")
    ),
    nest(
        (path = "/{id}/users", api = users::ApiDoc)
    ),
    components(
        schemas(
            Role,
            RoleCreate,
            RoleUpdate,
        )
    ),
    paths(
		find_all,
        create,
        update,
        delete
    ),
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("roles")
            .service(find_all)
            .service(create)
            .service(update)
            .service(delete)
            .configure(users::init_routes),
    );
}
