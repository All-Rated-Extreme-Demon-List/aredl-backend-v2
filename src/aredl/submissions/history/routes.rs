use std::sync::Arc;

use actix_web::{get, web, HttpResponse};
use utoipa::OpenApi;
use uuid::Uuid;

use crate::{
    aredl::submissions::history::SubmissionHistory, db::DbAppState, error_handler::ApiError,
};

#[utoipa::path(
    get,
    summary = "Get a submission's history",
    description = "Get the timestamps of each time this submission's status was changed.",
    tag = "AREDL - Submissions",
    responses(
        (status = 200, body = [SubmissionHistory])
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission")
    ),
)]
#[get("{id}/history")]
async fn get_history(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
) -> Result<HttpResponse, ApiError> {
    let history =
        web::block(move || SubmissionHistory::by_submission(db, id.into_inner())).await??;

    Ok(HttpResponse::Ok().json(history))
}

#[derive(OpenApi)]
#[openapi(components(schemas(SubmissionHistory)), paths(get_history,))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(get_history);
}
