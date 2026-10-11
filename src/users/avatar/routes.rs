use super::{AvatarRefresher, RefreshError};
use crate::app_data::db::DbAppState;
use crate::auth::{Permission, UserAuth};
use crate::error_handler::{ApiError, ErrorResponse};
use crate::users::User;
use actix_web::{post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    post,
    summary = "[Staff]Refresh Discord avatar",
    description = "Fetch the user's current Discord avatar and decoration.",
    tag = "Users - Avatar",
    params(("id" = String, Path, description = "The internal UUID, username or Discord ID of the user")),
    responses(
        (status = 200, body = User),
        (status = 400, description = "User has no linked Discord account", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse),
        (status = 429, description = "Refresh rate limited", body = ErrorResponse, headers(("Retry-After" = String, description = "Seconds until another refresh can be attempted"))),
        (status = 502, description = "Discord request failed", body = ErrorResponse)
    ),
    security(("bearer_token" = ["UserModify"]))
)]
#[post("/refresh", wrap = "UserAuth::require(Permission::UserModify)")]
async fn refresh(
    db: web::Data<Arc<DbAppState>>,
    refresher: web::Data<Arc<AvatarRefresher>>,
    id: web::Path<String>,
) -> Result<HttpResponse, RefreshError> {
    let db_user = db.get_ref().clone();
    let user_id = web::block(move || {
        User::from_str(&mut db_user.connection()?, &id.into_inner()).map(|user| user.id)
    })
    .await
    .map_err(ApiError::from)??;
    let user = refresher
        .refresh_user(db.get_ref().clone(), user_id)
        .await?;
    Ok(HttpResponse::Ok().json(user))
}

#[derive(OpenApi)]
#[openapi(
    tags((name = "Users - Avatar", description = "Endpoints for refreshing user Discord avatars")),
    paths(refresh)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/{id}/avatar").service(refresh));
}
