use crate::auth::{Authenticated, UserAuth};
use crate::clans::Clan;
use crate::db::DbAppState;
use crate::error_handler::ApiError;
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
        (status = 200)
    ),
	security(
		("access_token" = []),
		("api_key" = []),
	)
)]
#[post("/leave", wrap = "UserAuth::load()")]
async fn leave(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let result =
        web::block(move || Clan::leave(&mut db.connection()?, authenticated.user_id)).await??;
    Ok(HttpResponse::Ok().json(result))
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
