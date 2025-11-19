use crate::app_data::db::DbAppState;
use crate::arepl::submissions::pemonlist::PemonlistPlayer;
use crate::arepl::submissions::Submission;
use crate::auth::{Authenticated, UserAuth};
use crate::error_handler::ApiError;
use actix_web::{post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    post,
    summary = "[Auth]Sync with Pemonlist",
    description = "Import and/or update platformer submissions from a Pemonlist account. The pemonlist account must be linked to the same discord account as the authenticated user.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 200, body = Vec<Submission>)
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    )
)]
#[post("/sync", wrap = "UserAuth::load()")]
async fn sync_pemonlist(
    db: web::Data<Arc<DbAppState>>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let result = web::block(move || {
        PemonlistPlayer::sync_with_pemonlist(&mut db.connection()?, authenticated)
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
