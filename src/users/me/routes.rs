use crate::auth::{Authenticated, UserAuth};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::users::me::{clan, notifications, UserMeUpdate};
use crate::users::{User, UserResolved};
use actix_web::{get, patch, web, HttpResponse};
use std::sync::Arc;
use tracing_actix_web::RootSpan;
use utoipa::OpenApi;

#[utoipa::path(
    get,
    summary = "[Auth]Get authenticated user",
    description = "Get information about the currently authenticated user",
    tag = "Users - Me",
    responses(
        (status = 200, body = UserResolved)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[get("", wrap = "UserAuth::load()")]
async fn find(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let user = web::block(move || {
        let conn = &mut db.connection()?;
        User::find_me(conn, authenticated.user_id)
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
        (status = 200, body = User)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[patch("", wrap = "UserAuth::load()")]
async fn update(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
    user: web::Json<UserMeUpdate>,
    root_span: RootSpan,
) -> Result<HttpResponse, ApiError> {
    root_span.record("body", &tracing::field::debug(&user));
    let user = web::block(move || {
        let conn = &mut db.connection()?;
        User::update_me(conn, authenticated.user_id, user.into_inner())
    })
    .await??;
    Ok(HttpResponse::Ok().json(user))
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
            .service(find)
            .service(update),
    );
}
