use crate::app_data::db::DbAppState;
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::error_handler::ApiError;
use crate::users::User;
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

use super::{UserBadge, UserBadgeGrant};

#[utoipa::path(
    get,
    summary = "Get user badges",
    description = "Get all unlocked badges for a user.",
    tag = "Users - Badges",
    params(
        ("id" = String, description = "The internal UUID, username or discord ID of the user")
    ),
    responses(
        (status = 200, body = [UserBadge])
    ),
    security(
        ("access_token" = ["UserModify"]),
        ("api_key" = ["UserModify"]),
    )
)]
#[get("")]
async fn find_all(
    db: web::Data<Arc<DbAppState>>,
    user_id: web::Path<String>,
) -> Result<HttpResponse, ApiError> {
    let badges = web::block(move || {
        let conn = &mut db.connection()?;
        let user = User::from_str(conn, &user_id.into_inner())?;
        UserBadge::find_all(conn, user.id)
    })
    .await??;

    Ok(HttpResponse::Ok().json(badges))
}

#[utoipa::path(
    post,
    summary = "[Staff]Synchronize user badges",
    description = "Recalculate newly unlocked badges for a user.",
    tag = "Users - Badges",
    params(
        ("id" = Uuid, description = "The internal UUID of the user")
    ),
    responses(
        (status = 200, body = [UserBadge])
    ),
    security(
        ("access_token" = ["UserModify"]),
        ("api_key" = ["UserModify"]),
    )
)]
#[post("/sync", wrap = "UserAuth::require(Permission::UserModify)")]
async fn sync(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let badges = web::block(move || {
        let conn = &mut db.connection()?;
        authenticated.ensure_has_higher_privilege_than_user(conn, id.clone())?;
        let user_id = id.into_inner();
        UserBadge::update_user_badges(conn, user_id)?;
        UserBadge::find_all(conn, user_id)
    })
    .await??;

    Ok(HttpResponse::Ok().json(badges))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Grant user badges",
    description = "Grant the given badges to a user.",
    tag = "Users - Badges",
    params(
        ("id" = Uuid, description = "The internal UUID of the user")
    ),
    request_body = UserBadgeGrant,
    responses(
        (status = 200, body = [UserBadge])
    ),
    security(
        ("access_token" = ["UserModify"]),
        ("api_key" = ["UserModify"]),
    )
)]
#[patch("", wrap = "UserAuth::require(Permission::UserModify)")]
async fn grant(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    authenticated: Authenticated,
    badge: web::Json<UserBadgeGrant>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&badge));
    let badges = web::block(move || {
        let conn = &mut db.connection()?;
        authenticated.ensure_has_higher_privilege_than_user(conn, id.clone())?;
        let badges = UserBadge::grant(conn, id.into_inner(), badge.into_inner())?;
        Ok::<_, ApiError>(badges)
    })
    .await??;

    Ok(HttpResponse::Ok().json(badges))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Remove user badges",
    description = "Remove the given badges from a user.",
    tag = "Users - Badges",
    params(
        ("id" = Uuid, description = "The internal UUID of the user")
    ),
    request_body = [String],
    responses(
        (status = 200, body = [UserBadge])
    ),
    security(
        ("access_token" = ["UserModify"]),
        ("api_key" = ["UserModify"]),
    )
)]
#[delete("", wrap = "UserAuth::require(Permission::UserModify)")]
async fn remove(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    authenticated: Authenticated,
    badge_codes: web::Json<Vec<String>>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&badge_codes));
    let badges = web::block(move || {
        let conn = &mut db.connection()?;
        authenticated.ensure_has_higher_privilege_than_user(conn, id.clone())?;
        let badges = UserBadge::remove_all(conn, id.into_inner(), badge_codes.into_inner())?;
        Ok::<_, ApiError>(badges)
    })
    .await??;

    Ok(HttpResponse::Ok().json(badges))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Users - Badges", description = "Staff endpoints for fetching and managing user badges")
    ),
    components(
        schemas(
            UserBadge,
            UserBadgeGrant,
        )
    ),
    paths(
        find_all,
        sync,
        grant,
        remove,
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/{id}/badges")
            .service(find_all)
            .service(sync)
            .service(grant)
            .service(remove),
    );
}
