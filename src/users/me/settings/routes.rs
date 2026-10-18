use crate::app_data::db::DbAppState;
use crate::auth::{Authenticated, UserAuth};
use crate::error_handler::{ApiError, ErrorResponse};
use crate::users::me::settings::{ClanNotifications, UserSettings, UserSettingsUpdate};
use actix_web::{get, patch, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "[Auth]Get my settings",
    description = "Get the authenticated user's settings.",
    tag = "Users - Me",
    responses((status = 200, body = UserSettings)),
    security(("bearer_token" = [])),
)]
#[get("", wrap = "UserAuth::load()")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let settings = db
        .run(move |conn| UserSettings::find(conn, authenticated.user_id))
        .await?;
    Ok(HttpResponse::Ok().json(settings))
}

#[utoipa::path(
    patch,
    summary = "[Auth]Update my settings",
    description = "Change the authenticated user's settings.",
    tag = "Users - Me",
    request_body = UserSettingsUpdate,
    responses(
        (status = 200, body = UserSettings),
        (status = 400, description = "No changes were provided or invalid settings", body = ErrorResponse),
    ),
    security(("bearer_token" = [])),
)]
#[patch("", wrap = "UserAuth::load()")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
    settings: web::Json<UserSettingsUpdate>,
) -> Result<HttpResponse, ApiError> {
    let settings = db
        .run(move |conn| UserSettings::update(conn, authenticated.user_id, &settings))
        .await?;
    Ok(HttpResponse::Ok().json(settings))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(UserSettings, UserSettingsUpdate, ClanNotifications)),
    paths(find, update)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/settings").service(find).service(update));
}
