use crate::app_data::db::DbAppState;
use crate::audit::AuditLogEntry;
use crate::auth::{Authenticated, Permission, UserAuth};
use crate::cache_control::CacheController;
use crate::error_handler::{ApiError, ErrorResponse};
use crate::page_helper::{PageQuery, Paginated};
use crate::users::{
    avatar, badges, me, merge, names, AccountStatus, PlaceholderOptions, User,
    UserAccountStatusUpdate, UserListQueryOptions, UserPage, UserResolved, UserUpdate,
};
use actix_web::{delete, get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "[AuthPublic]Get user",
    description = "Get a specific user by their internal UUID, username or discord ID. The UUID of a user who has been merged will resolve to the user they were merged into.",
    tag = "Users",
    params(
        ("id" = String, Path, description = "The internal UUID, username or discord ID of the user to lookup"),
    ),
    responses(
        (status = 200, body = UserResolved),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security((), ("bearer_token" = [])),
)]
#[get(
    "/{id}",
    wrap = "UserAuth::load()",
    wrap = "CacheController::auth_public_with_max_age(300)"
)]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<String>,
    authenticated: Option<Authenticated>,
) -> Result<HttpResponse, ApiError> {
    let result = db
        .run(move |conn| {
            UserResolved::from_str(conn, id.into_inner().as_str(), authenticated.as_ref())
        })
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    get,
    summary = "[AuthPublic]Get users",
    description = "Get paginated list of users",
    tag = "Users",
    params(
        ("account_status" = Option<AccountStatus>, Query, description = "The account status to filter by"),
        PageQuery<100>,
        ("name_filter" = Option<String>, Query, description = "The search filter to apply. Uses the SQL LIKE operator syntax."),
        ("placeholder" = Option<bool>, Query, description = "If specified, will only fetch users that are/are not placeholders. If not, all types of users are returned.")
    ),
    responses(
        (status = 200, body = Paginated<UserPage>),
    ),
    security((), ("bearer_token" = [])),
)]
#[get(
    "",
    wrap = "UserAuth::load()",
    wrap = "CacheController::auth_public_with_max_age(300)"
)]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<100>>,
    options: web::Query<UserListQueryOptions>,
    authenticated: Option<Authenticated>,
) -> Result<HttpResponse, ApiError> {
    let result = db
        .run(move |conn| {
            User::find_all(
                conn,
                page_query.into_inner(),
                &options.into_inner(),
                authenticated.as_ref(),
            )
        })
        .await?;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add placeholder",
    description = "Creates a new placeholder user",
    tag = "Users",
    request_body = PlaceholderOptions,
    responses(
        (status = 201, body = User),
    ),
    security(("bearer_token" = ["PlaceholderCreate"])),
)]
#[post(
    "/placeholders",
    wrap = "UserAuth::require(Permission::PlaceholderCreate)"
)]
async fn create_placeholder(
    db: web::Data<Arc<DbAppState>>,
    options: web::Json<PlaceholderOptions>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&options));
    let result = db
        .transaction(move |conn| {
            let created_user = User::create_placeholder(conn, options.into_inner())?;
            AuditLogEntry::log_create(conn, Some(authenticated.user_id), &created_user)?;
            Ok::<_, ApiError>(created_user)
        })
        .await?;
    Ok(HttpResponse::Created().json(result))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit user",
    description = "Edit a user's base information. Your privilege level needs to be strictly higher than the one of the user you are trying to edit.",
    tag = "Users",
    params(
        ("id" = String, description = "The internal UUID, username or discord ID of the user")
    ),
    request_body = UserUpdate,
    responses(
        (status = 200, body = User),
        (status = 403, description = "You do not have sufficient privilege to affect this user, or the user has been deleted", body = ErrorResponse, examples(
            ("user_privilege" = (value = json!({"message": "You do not have sufficient privilege to affect this user."})))
        )),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["UserModify"])),
)]
#[patch("/{id}", wrap = "UserAuth::require(Permission::UserModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<String>,
    user: web::Json<UserUpdate>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&user));
    let result = db
        .transaction(move |conn| {
            let user_id = User::from_str(conn, id.into_inner().as_str())?.id;
            authenticated.ensure_has_higher_privilege_than_user(conn, user_id)?;
            let before = User::from_uuid(conn, user_id)?;
            let updated_user = User::update(conn, user_id, &user.into_inner())?;
            AuditLogEntry::log_update(conn, Some(authenticated.user_id), &before, &updated_user)?;
            Ok::<_, ApiError>(updated_user)
        })
        .await?;

    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit account status",
    description = "Edit a user's account status. Your privilege level needs to be strictly higher than the one of the user you are trying to edit.    \n\
        \n\
        | Value | Account status |    \n\
        |---|---|    \n\
        | 0 | Normal (No restriction)|    \n\
        | 1 | Semi-unranked (Does not appear on user leaderboards, but still contributes records to country and clan leaderboards)|    \n\
        | 2 | Unranked (Does not want to appear on leaderboards, but is still able to submit records)|    \n\
        | 3 | List banned (Has been banned from the list. Does not appear on leaderboards, and isn't able to submit records)|    \n\
        | 4 | Redacted (Hidden users that have been removed from the site for various reasons, but whose account are kept for internal purposes)|    \n\
        | 5 | Deleted (Accounts that have been removed from the site. Cannot be manually set through this endpoint.)|    \n\
    ",
    tag = "Users",
    params(
        ("id" = String, description = "The internal UUID, username or discord ID of the user")
    ),
    request_body = UserAccountStatusUpdate,
    responses(
        (status = 200, body = User),
        (status = 400, description = "Account status cannot be manually set to deleted, use the delete endpoint instead.", body = ErrorResponse),
        (status = 403, description = "Insufficient privilege to affect this user, missing UserRedact permission, or the user has been deleted", body = ErrorResponse, examples(
            ("user_privilege" = (value = json!({"message": "You do not have sufficient privilege to affect this user."}))),
            ("redact_permission" = (value = json!({"message": "You do not have the required permission (UserRedact) to perform this action"})))
        )),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["UserBan"])),
)]
#[patch("/{id}/ban", wrap = "UserAuth::require(Permission::UserBan)")]
async fn update_account_status(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<String>,
    user: web::Json<UserAccountStatusUpdate>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", tracing::field::debug(&user));
    let result = db
        .transaction(move |conn| {
            let user_id = User::from_str(conn, id.into_inner().as_str())?.id;
            authenticated.ensure_has_higher_privilege_than_user(conn, user_id)?;
            if user.account_status == AccountStatus::Redacted {
                authenticated.ensure_has_permission(conn, Permission::UserRedact)?;
            }
            let before = User::from_uuid(conn, user_id)?;
            let updated_user = User::update_account_status(
                conn,
                &authenticated,
                user_id,
                user.into_inner().account_status,
            )?;
            AuditLogEntry::log_update(conn, Some(authenticated.user_id), &before, &updated_user)?;
            Ok::<_, ApiError>(updated_user)
        })
        .await?;

    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    delete,
    summary = "[Staff]Delete user",
    description = "Soft delete a user: anonymize their profile, clear Discord data and connected accounts, invalidate their tokens, delete their records, and deny their submissions. Sets account status to 5 (Deleted). If they have verifications, those will be kept, but the account will be converted to a placeholder one. Your privilege level must be strictly higher than the target user's.",
    tag = "Users",
    params(("id" = String, Path, description = "The internal UUID, username or Discord ID of the user")),
    responses(
        (status = 204, description = "User deleted"),
        (status = 403, description = "Insufficient permission or privilege to affect this user", body = ErrorResponse),
        (status = 404, description = "User not found", body = ErrorResponse)
    ),
    security(("bearer_token" = ["UserDelete"])),
)]
#[delete("/{id}", wrap = "UserAuth::require(Permission::UserDelete)")]
async fn delete(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<String>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    db.run(move |conn| {
        let user_id = User::from_str(conn, id.into_inner().as_str())?.id;
        authenticated.ensure_has_higher_privilege_than_user(conn, user_id)?;
        User::soft_delete(conn, user_id, &authenticated)
    })
    .await?;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/{id}/avatar", api = avatar::ApiDoc),
        (path = "/{id}/badges", api = badges::ApiDoc),
        (path = "/names", api = names::ApiDoc),
        (path = "/@me", api = me::ApiDoc),
        (path = "/merge", api = merge::ApiDoc),
    ),
    components(
        schemas(
            User,
            PlaceholderOptions,
            UserUpdate,
            UserAccountStatusUpdate,
        )
    ),
    paths(
        find,
        list,
        create_placeholder,
        update,
        update_account_status,
        delete
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/users")
            .configure(badges::init_routes)
            .configure(me::init_routes)
            .configure(avatar::init_routes)
            .configure(names::init_routes)
            .configure(merge::init_routes)
            .service(find)
            .service(list)
            .service(create_placeholder)
            .service(update)
            .service(update_account_status)
            .service(delete),
    );
}
