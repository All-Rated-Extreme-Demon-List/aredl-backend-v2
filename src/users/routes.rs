use std::sync::Arc;
use uuid::Uuid;
use actix_web::{get, patch, post, web, HttpResponse};
use crate::auth::{UserAuth, Permission, Authenticated, check_higher_privilege};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::page_helper::PageQuery;
use crate::users::{me, names, PlaceholderOptions, User, UserUpdate, UserBanUpdate, UserListQueryOptions};

#[get("")]
async fn list(
    db: web::Data<Arc<DbAppState>>,
    page_query: web::Query<PageQuery<100>>,
    options: web::Query<UserListQueryOptions>
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        User::find(&mut conn, page_query.into_inner(), options.into_inner())
    }).await??;
    Ok(HttpResponse::Ok().json(result))
}

#[post("placeholders", wrap="UserAuth::require(Permission::PlaceholderCreate)")]
async fn create_placeholder(
    db: web::Data<Arc<DbAppState>>,
    options: web::Json<PlaceholderOptions>,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        User::create_placeholder(&mut conn, options.into_inner())
    }).await??;
    Ok(HttpResponse::Ok().json(result))
}

#[patch("/{id}", wrap = "UserAuth::require(Permission::UserModify)")]
async fn update(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, user: web::Json<UserUpdate>, authenticated: Authenticated, ) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        check_higher_privilege(db.clone(), authenticated.user_id, id.clone())?;
        let mut conn = db.connection()?;
        User::update(&mut conn, id.into_inner(), user.into_inner())
    }).await??;

    Ok(HttpResponse::Ok().json(result))
}

#[patch("/{id}/ban", wrap = "UserAuth::require(Permission::UserBan)")]
async fn ban(db: web::Data<Arc<DbAppState>>, id: web::Path<Uuid>, user: web::Json<UserBanUpdate>, authenticated: Authenticated) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        check_higher_privilege(db.clone(), authenticated.user_id, id.clone())?;
        let mut conn = db.connection()?;
        User::ban(&mut conn, id.into_inner(), user.into_inner().ban_level)
    }).await??;

    Ok(HttpResponse::Ok().json(result))
}

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/users")
            .configure(me::init_routes)
            .configure(names::init_routes)
            .service(list)
            .service(create_placeholder)
            .service(update)
            .service(ban)
    );
}