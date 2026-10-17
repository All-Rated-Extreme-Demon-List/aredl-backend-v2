use crate::app_data::db::DbAppState;
use crate::audit::AuditLogEntry;
use crate::auth::{Authenticated, UserAuth};
use crate::error_handler::{ApiError, ErrorResponse};
use crate::users::avatar::{AvatarRefresher, RefreshError};
use crate::users::badges::UserBadge;
use crate::users::me::{clan, notifications, UserMeUpdate};
use crate::users::{User, UserResolved};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "[Auth]Get authenticated user",
    description = "Get information about the currently authenticated user",
    tag = "Users - Me",
    responses(
        (status = 200, body = UserResolved),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(("bearer_token" = [])),
)]
#[get("", wrap = "UserAuth::load()")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let user = db
        .run(move |conn| {
            let user = User::find(conn, authenticated.user_id)?;
            UserResolved::from_user(conn, user, Some(&authenticated))
        })
        .await?;
    Ok(HttpResponse::Ok().json(user))
}

#[utoipa::path(
    patch,
    summary = "[Auth]Edit authenticated user",
    description = "Update the current authenticated user base information",
    tag = "Users - Me",
    request_body = UserMeUpdate,
    responses(
        (status = 200, body = User),
        (status = 400, description = "Display name or description is too long, account status is invalid, country changes are on cooldown, or the selected level or badge is not unlocked", body = ErrorResponse, examples(
            ("display_name_too_long" = (value = json!({"message": "The display name can at most be 35 characters long."}))),
            ("description_too_long" = (value = json!({"message": "The description can at most be 300 characters long."}))),
            ("country_cooldown" = (value = json!({"message": "You have recently changed your country, please wait 30 days and 0 hours before changing it again."}))),
            ("background_level" = (value = json!({"message": "You have not beaten the selected level."}))),
            ("featured_badge" = (value = json!({"message": "You have not unlocked the selected badge."})))
        )),
        (status = 403, description = "You cannot change your account status while banned from the list", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(("bearer_token" = [])),
)]
#[patch("", wrap = "UserAuth::load()")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
    user: web::Json<UserMeUpdate>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&user));
    let user = db
        .transaction(move |conn| {
            let before = User::find(conn, authenticated.user_id)?;
            let updated_user = User::update_me(conn, &before, &user.into_inner())?;
            AuditLogEntry::log_update(conn, Some(authenticated.user_id), &before, &updated_user)?;
            Ok::<_, ApiError>(updated_user)
        })
        .await?;
    Ok(HttpResponse::Ok().json(user))
}

#[utoipa::path(
    post,
    summary = "[Auth]Synchronize my badges",
    description = "Recalculate newly unlocked badges for the authenticated user.",
    tag = "Users - Me",
    responses(
        (status = 200, body = [UserBadge]),
    ),
    security(("bearer_token" = [])),
)]
#[post("/sync", wrap = "UserAuth::load()")]
async fn sync(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let badges = db
        .run(move |conn| {
            UserBadge::update_user_badges(conn, authenticated.user_id)?;
            UserBadge::find_all(conn, authenticated.user_id)
        })
        .await?;

    Ok(HttpResponse::Ok().json(badges))
}

#[utoipa::path(
    post,
    summary = "[Auth]Refresh my Discord avatar",
    description = "Fetch the authenticated user's current Discord avatar and decoration. ",
    tag = "Users - Me",
    responses(
        (status = 200, body = User),
        (status = 400, description = "User has no linked Discord account", body = ErrorResponse),
        (status = 429, description = "Refresh rate limited", body = ErrorResponse, headers(("Retry-After" = String, description = "Seconds until another refresh can be attempted"))),
        (status = 502, description = "Discord request failed", body = ErrorResponse)
    ),
    security(("bearer_token" = []))
)]
#[post("/avatar/refresh", wrap = "UserAuth::load()")]
async fn refresh_avatar(
    db: web::Data<Arc<DbAppState>>,
    refresher: web::Data<Arc<AvatarRefresher>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, RefreshError> {
    let user = refresher
        .refresh_user(db.get_ref().clone(), authenticated.user_id)
        .await?;
    Ok(HttpResponse::Ok().json(user))
}

#[utoipa::path(
    delete,
    summary = "[Auth]Delete authenticated user",
    description = "Soft delete the authenticated user: anonymize their profile, clear Discord data and connected accounts, invalidate their tokens, delete their records, and deny their submissions. Sets account status to 5 (Deleted). If they have verifications, those will be kept, and the account will be converted to a placeholder one.",
    tag = "Users - Me",
    responses((status = 204, description = "User deleted")),
    security(("bearer_token" = [])),
)]
#[delete("", wrap = "UserAuth::load()")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    db.run(move |conn| User::soft_delete(conn, authenticated.user_id, &authenticated))
        .await?;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/clan", api = clan::ApiDoc),
		(path = "/notifications", api = notifications::ApiDoc)
    ),
    components(
        schemas(
            UserResolved,
            UserMeUpdate,
        )
    ),
    paths(
        sync,
        find,
        update,
        refresh_avatar,
        delete,
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/@me")
            .configure(clan::init_routes)
            .configure(notifications::init_routes)
            .service(sync)
            .service(refresh_avatar)
            .service(find)
            .service(update)
            .service(delete),
    );
}
