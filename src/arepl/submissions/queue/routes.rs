use crate::{
    arepl::submissions::{
        queue::{QueuePositionResponse, SubmissionQueue},
        Submission,
    },
    auth::{Authenticated, UserAuth},
    app_data::db::DbAppState,
    error_handler::ApiError,
};
use actix_web::{get, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;
use uuid::Uuid;

#[utoipa::path(
    get,
    summary = "[Auth]Get queue position for a submission",
    description = "Returns the position of a specific submission in the pending queue.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 200, description = "Queue position found", body = QueuePositionResponse),
        (status = 404, description = "Submission not found or not pending"),
    ),
    security(
        ("access_token" = []),
        ("api_key" = []),
    ),
    params(
        ("id" = Uuid, description = "The ID of the submission to check position for")
    )
)]
#[get("{id}/queue", wrap = "UserAuth::load()")]
async fn get_queue_position(
    db: web::Data<Arc<DbAppState>>,
    id: web::Path<Uuid>,
    _auth: Authenticated,
) -> Result<HttpResponse, ApiError> {
    let (position, total) =
        web::block(move || Submission::get_queue_position(&mut db.connection()?, id.into_inner()))
            .await??;

    Ok(HttpResponse::Ok().json(QueuePositionResponse { position, total }))
}

#[utoipa::path(
    get,
    summary = "Get submissions queue",
    description = "Get the amount of pending submissions.",
    tag = "AREDL (P) - Submissions",
    responses(
        (status = 200, body = SubmissionQueue)
    )
)]
#[get("queue")]
async fn get_queue(db: web::Data<Arc<DbAppState>>) -> Result<HttpResponse, ApiError> {
    let queue = web::block(move || SubmissionQueue::get_queue(&mut db.connection()?)).await??;
    Ok(HttpResponse::Ok().json(queue))
}

#[derive(OpenApi)]
#[openapi(
    components(schemas(SubmissionQueue, QueuePositionResponse,)),
    paths(get_queue, get_queue_position,)
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(get_queue).service(get_queue_position);
}
