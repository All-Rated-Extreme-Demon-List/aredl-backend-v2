use crate::list::submissions::routes::SubmissionPath;
use crate::list::List;
use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use utoipa::OpenApi;

use crate::{
    app_data::db::DbAppState,
    auth::{Authenticated, UserAuth},
    error_handler::ApiError,
    list::submissions::history::SubmissionHistoryResolved,
};

use super::SubmissionHistoryOptions;

#[utoipa::path(
    get,
    summary = "Get a submission's history",
    description = "Get the timestamps of each time this submission's status was changed.",
    tag = "List - Submissions",
    responses(
        (status = 200, body = [SubmissionHistoryResolved]),
    ),
    params(
        ("list" = List, Path, description = "The selected list (classic / aredl or platformer / arepl)"),
        ("id" = Uuid, description = "The ID of the submission")
    ),
    security(("bearer_token" = [])),
)]
#[get("{id}/history", wrap = "UserAuth::load()")]
async fn get_history(
    db: web::Data<Arc<DbAppState>>,
    path: web::Path<SubmissionPath>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let history = web::block(move || {
        SubmissionHistoryResolved::by_submission_id(
            &mut db.connection()?,
            path.list,
            path.id,
            &authenticated,
        )
    })
    .await??;

    Ok(HttpResponse::Ok().json(history))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(SubmissionHistoryResolved, SubmissionHistoryOptions)),
    paths(get_history)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(get_history);
}
