use std::sync::Arc;
use actix_web::{get, patch, HttpResponse, web};
use utoipa::OpenApi;
use crate::auth::{UserAuth, Authenticated};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::users::model::{User, UserResolved};
use crate::users::me::model::UserMeUpdate;

#[utoipa::path(
    get,
    summary = "[Auth]Get authenticated user",
    description = "Get information about the currently authenticated user",
    tag = "Users",
    responses(
        (status = 200, body = UserResolved)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[get("", wrap="UserAuth::load()")]
async fn find(db: web::Data<Arc<DbAppState>>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    let user = web::block(
        move || {
            let conn = &mut db.connection()?;
            User::find_me(conn, authenticated.user_id)
        }
    ).await??;
    Ok(HttpResponse::Ok().json(user))
}

#[utoipa::path(
    patch,
    summary = "[Auth]Edit authenticated user",
    description = "Update the current authenticated user base information",
    tag = "Users",
    request_body = UserMeUpdate,
    responses(
        (status = 200, body = UserResolved)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[patch("", wrap = "UserAuth::load()")]
async fn update(db: web::Data<Arc<DbAppState>>, authenticated: Authenticated, user: web::Json<UserMeUpdate>) -> Result<HttpResponse, ApiError> {
    let user = web::block(
        move || {
            let conn = &mut db.connection()?;
            User::update_me(conn, authenticated.user_id, user.into_inner())
        }
    ).await??;
    Ok(HttpResponse::Ok().json(user))
}

#[derive(OpenApi)]
#[openapi(
    components(
        schemas(
            UserResolved,
            UserMeUpdate,
        )
    ),
    paths(
        find,
        update,
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/@me")
            .service(find)
            .service(update)
    );
}