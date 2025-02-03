use std::sync::Arc;
use actix_web::{get, delete, HttpResponse, patch, post, web};
use utoipa::OpenApi;
use crate::roles::{Role, RoleCreate, RoleUpdate, users};
use crate::auth::{UserAuth, Permission};
use crate::db::DbAppState;
use crate::error_handler::ApiError;


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
	let roles = web::block(
		|| Role::find_all(db)
	).await??;
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
#[post("", wrap="UserAuth::require(Permission::RoleManage)")]
async fn create(db: web::Data<Arc<DbAppState>>, role: web::Json<RoleCreate>) -> Result<HttpResponse, ApiError> {
    let role = web::block(
        || Role::create(db, role.into_inner())
    ).await??;
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
#[patch("/{id}", wrap="UserAuth::require(Permission::RoleManage)")]
async fn update(db: web::Data<Arc<DbAppState>>, id: web::Path<i32>, role: web::Json<RoleUpdate>) -> Result<HttpResponse, ApiError> {
    let role = web::block(
        || Role::update(db, id.into_inner(), role.into_inner())
    ).await??;
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
#[delete("/{id}", wrap="UserAuth::require(Permission::RoleManage)")]
async fn delete(db: web::Data<Arc<DbAppState>>, id: web::Path<i32>) -> Result<HttpResponse, ApiError> {
    let role = web::block(
        || Role::delete(db, id.into_inner())
    ).await??;
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
            .configure(users::init_routes)
    );
}