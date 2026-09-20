use crate::app_data::db::DbAppState;
use crate::arepl::submissions::pemonlist::PemonlistPlayer;
use crate::arepl::submissions::Submission;
use crate::auth::{Authenticated, UserAuth};
use crate::error_handler::{ApiError, ErrorResponse};
use actix_web::{post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    post,
    summary = "[Auth]Sync with Pemonlist",
    description = "Import and/or update platformer submissions from a Pemonlist account. The pemonlist account must be linked to the same discord account as the authenticated user.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 200, body = Vec<Submission>),
        (status = 404, description = "User or Pemonlist player not found", body = ErrorResponse),
        (status = 422, description = "Your account is not linked to a Discord account", body = ErrorResponse),
        (status = 502, description = "Failed to request Pemonlist data, parse its response or parse a completion time", body = ErrorResponse, examples(
            ("request_failed" = (value = json!({"message": "error sending request for url (https://pemonlist.com/api/player/123456789)"}))),
            ("invalid_response" = (value = json!({"message": "Failed to parse data received from pemonlist: error decoding response body"}))),
            ("invalid_completion_time" = (value = json!({"message": "Malformed formatted_time (out of range)"})))
        ))
    ),
    security(("bearer_token" = [])),
)]
#[post("/sync", wrap = "UserAuth::load()")]
async fn sync_pemonlist(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        PemonlistPlayer::sync_with_pemonlist(&mut db.connection()?, &authenticated)
    })
    .await??;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL (P) - Submissions", description = "Endpoints for fetching and managing platformer submissions")
    ),
    paths(
		sync_pemonlist
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/pemonlist").service(sync_pemonlist));
}
