use crate::auth::{check_higher_privilege, Authenticated, Permission, UserAuth};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::users::{
    me, merge, names, PlaceholderOptions, User, UserBanUpdate, UserListQueryOptions, UserPage,
    UserUpdate,
};
use actix_web::{get, patch, post, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "Get users",
    description = "Get paginated list of users",
    tag = "Users",
    params(
        ("page" = Option<i64>, Query, description = "The page of the users list to fetch"),
        ("per_page" = Option<i64>, Query, description = "The number of users to fetch per page"),
        ("name_filter" = Option<String>, Query, description = "The search filter to apply. Uses the SQL LIKE operator syntax."),
        ("placeholder" = Option<bool>, Query, description = "If specified, will only fetch users that are/are not placeholders. If not, all types of users are returned.")
    ),
    responses(
        (status = 200, body = Paginated<UserPage>)
    ),
)]
#[get("")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<100>>,
    options: web::Query<UserListQueryOptions>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        User::find(&mut conn, page_query.into_inner(), options.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    post,
    summary = "[Staff]Add placeholder",
    description = "Creates a new placeholder user",
    tag = "Users",
    request_body = PlaceholderOptions,
    responses(
        (status = 200, body = User)
    ),
    security(
        ("access_token" = ["PlaceholderCreate"]),
    )
)]
#[post(
    "placeholders",
    wrap = "UserAuth::require(Permission::PlaceholderCreate)"
)]
async fn create_placeholder(
    db: web::Data<Arc<DbAppState>>,
    options: web::Json<PlaceholderOptions>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&options));
    let result = web::block(move || {
        let mut conn = db.connection()?;
        User::create_placeholder(&mut conn, options.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit user",
    description = "Edit a user's base information. Your privilege level needs to be strictly higher than the one of the user you are trying to edit.",
    tag = "Users",
    params(
        ("id" = Uuid, description = "The internal UUID of the user")
    ),
    request_body = UserUpdate,
    responses(
        (status = 200, body = User)
    ),
    security(
        ("access_token" = ["UserModify"]),
        ("api_key" = ["UserModify"]),
    )
)]
#[patch("/{id}", wrap = "UserAuth::require(Permission::UserModify)")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    user: web::Json<UserUpdate>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&user));
    let result = web::block(move || {
        check_higher_privilege(db.clone(), authenticated.user_id, id.clone())?;
        let mut conn = db.connection()?;
        User::update(&mut conn, id.into_inner(), user.into_inner())
    })
    .await??;

    Ok(HttpResponse::Ok().json(result))
}

#[utoipa::path(
    patch,
    summary = "[Staff]Edit ban level",
    description = "Edit a user's ban level. Your privilege level needs to be strictly higher than the one of the user you are trying to edit.    \n\
        \n\
        | Ban level | Account status |    \n\
        |---|---|    \n\
        | 0 | Normal (No restriction)|    \n\
        | 1 | Unranked (Does not want to appear on the leaderboards, but is still able to submit records)|    \n\
        | 2 | List banned (Has been banned from the list. Does not appear on leaderboards, and isn't able to submit records)|    \n\
        | 3 | Redacted (Hidden users that have been removed from the site for various reasons, but whose account are kept for internal purposes)|    \n\
    ",
    tag = "Users",
    params(
        ("id" = Uuid, description = "The internal UUID of the user")
    ),
    request_body = UserBanUpdate,
    responses(
        (status = 200, body = User)
    ),
    security(
        ("access_token" = ["UserBan"]),
        ("api_key" = ["UserBan"]),
    )
)]
#[patch("/{id}/ban", wrap = "UserAuth::require(Permission::UserBan)")]
async fn ban(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    user: web::Json<UserBanUpdate>,
    authenticated: Authenticated,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&user));
    let result = web::block(move || {
        check_higher_privilege(db.clone(), authenticated.user_id, id.clone())?;
        let mut conn = db.connection()?;
        User::ban(&mut conn, id.into_inner(), user.into_inner().ban_level)
    })
    .await??;

    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/names", api = names::ApiDoc),
        (path = "/@me", api = me::ApiDoc),
        (path = "/merge", api = merge::ApiDoc),
    ),
    components(
        schemas(
            User,
            PlaceholderOptions,
            UserUpdate,
            UserBanUpdate,
        )
    ),
    paths(
        list,
        create_placeholder,
        update,
        ban
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/users")
            .configure(me::init_routes)
            .configure(names::init_routes)
            .configure(merge::init_routes)
            .service(list)
            .service(create_placeholder)
            .service(update)
            .service(ban),
    );
}
