use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    arepl::submissions::history::SubmissionHistoryResolved,
    auth::{Authenticated, UserAuth},
    app_data::db::DbAppState,
    error_handler::ApiError,
};

use super::SubmissionHistoryOptions;

#[utoipa::path(
    get,
    summary = "Get a submission's history",
    description = "Get the timestamps of each time this submission's status was changed.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 200, body = [SubmissionHistoryResolved])
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
)]
#[get("{id}/history", wrap = "UserAuth::load()")]
async fn get_history(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    options: web::Query<SubmissionHistoryOptions>,
    authenticated: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let history = web::block(move || {
        SubmissionHistoryResolved::by_submission_id(
            &mut db.connection()?,
            id.into_inner(),
            options.into_inner(),
            authenticated,
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
