use crate::app_data::db::DbAppState;
use crate::auth::{Authenticated, UserAuth};
use crate::error_handler::{ApiError, ErrorResponse};
use crate::users::badges::UserBadge;
use crate::users::me::{clan, notifications, UserMeUpdate};
use crate::users::{User, UserResolved};
use actix_web::{get, patch, post, web, HttpResponse};
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
    let user = web::block(move || {
        UserResolved::from_uuid(
            &mut db.connection()?,
            authenticated.user_id,
            Some(&authenticated),
        )
    })
    .await??;
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
        (status = 400, description = "Display name or description is too long, country changes are on cooldown, or the selected level or badge is not unlocked", body = ErrorResponse, examples(
            ("display_name_too_long" = (value = json!({"message": "The display name can at most be 35 characters long."}))),
            ("description_too_long" = (value = json!({"message": "The description can at most be 300 characters long."}))),
            ("country_cooldown" = (value = json!({"message": "You have recently changed your country, please wait 30 days and 0 hours before changing it again."}))),
            ("background_level" = (value = json!({"message": "You have not beaten the selected level."}))),
            ("featured_badge" = (value = json!({"message": "You have not unlocked the selected badge."})))
        )),
        (status = 403, description = "You cannot change your ban level while banned from the list", body = ErrorResponse),
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
    let user = web::block(move || {
        User::update_me(
            &mut db.connection()?,
            authenticated.user_id,
            &user.into_inner(),
        )
    })
    .await??;
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
    let badges = web::block(move || {
        let conn = &mut db.connection()?;
        UserBadge::update_user_badges(conn, authenticated.user_id)?;
        UserBadge::find_all(conn, authenticated.user_id)
    })
    .await??;

    Ok(HttpResponse::Ok().json(badges))
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
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/@me")
            .configure(clan::init_routes)
            .configure(notifications::init_routes)
            .service(sync)
            .service(find)
            .service(update),
    );
}
