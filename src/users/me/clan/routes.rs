use crate::app_data::db::DbAppState;
use crate::auth::{Authenticated, UserAuth};
use crate::clans::Clan;
use crate::error_handler::{ApiError, ErrorResponse};
use crate::users::me::clan::invites;
use actix_web::{post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    post,
    summary = "[Auth]Leave clan",
    description = "Leaves the clan you are currently in.",
    tag = "Users - Me",
    responses(
        (status = 204),
        (status = 404, description = "You are not a member of a clan", body = ErrorResponse),
        (status = 409, description = "You cannot leave a clan you own", body = ErrorResponse)
    ),
    security(("bearer_token" = [])),
)]
#[post("/leave", wrap = "UserAuth::load()")]
async fn leave(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    web::block(move || Clan::leave(&mut db.connection()?, authenticated.user_id)).await??;
    Ok(HttpResponse::NoContent().finish())
}

#[derive(OpenApi)]
#[openapi(
	nest(
		(path = "/invites", api = invites::ApiDoc)
	),
    paths(
        leave
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/clan")
            .configure(invites::init_routes)
            .service(leave),
    );
}
