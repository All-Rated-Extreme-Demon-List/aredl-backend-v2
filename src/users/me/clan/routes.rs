use std::sync::Arc;
use actix_web::{post, web, HttpResponse};
use utoipa::OpenApi;
use crate::auth::{UserAuth, Authenticated};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::clans::Clan;
use crate::users::me::clan::invites;

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
#[post("", wrap="UserAuth::load()")]
async fn leave(
    db: web::Data<Arc<DbAppState>>,
	authenticated: Authenticated
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        let mut conn = db.connection()?;
        Clan::leave(&mut conn, authenticated.user_id)
    }).await??;
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
            .service(leave)
    );
}