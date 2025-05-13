use crate::arepl::records::pemonlist::PemonlistPlayer;
use crate::arepl::records::Record;
use crate::auth::{Authenticated, UserAuth};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use actix_web::{post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

#[utoipa::path(
    post,
    summary = "[Auth]Sync with Pemonlist",
    description = "Import and/or update platformer records from a Pemonlist account. The pemonlist account must be linked to the same discord account as the authenticated user.",
    tag = "AREDL (P) - Records",
    responses(
        (status = 200, body = Vec<Record>)
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
    let result = PemonlistPlayer::sync_with_pemonlist(db, authenticated).await?;
    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL (P) - Records", description = "Endpoints for fetching and managing platformer records")
    ),
    paths(
		sync_pemonlist
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/pemonlist").service(sync_pemonlist));
}
