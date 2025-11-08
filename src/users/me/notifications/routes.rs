use crate::auth::{Authenticated, UserAuth};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::users::me::notifications::Notification;
use actix_web::{get, post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "[Auth]Get my notifications",
    description = "Get the list of notifications you've received",
    tag = "Users - Me",
    responses(
        (status = 200, body = [Notification])
    ),
	security(
		("access_token" = []),
		("api_key" = []),
	)
)]
#[get("", wrap = "UserAuth::load()")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        Notification::find_all_me_notifications(&mut db.connection()?, authenticated.user_id)
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Auth]Clear all notifications",
    description = "Removes all your current notifications.",
    tag = "Users - Me",
    responses(
        (status = 200)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[post("/clear", wrap = "UserAuth::load()")]
async fn clear(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        Notification::clear_me_notifications(&mut db.connection()?, authenticated.user_id)
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(components(schemas(Notification)), paths(list, clear))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/notifications").service(list).service(clear));
}
