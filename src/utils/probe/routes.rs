use actix_web::{post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

use crate::{
    auth::{Permission, UserAuth},
    drive::DriveState,
    error_handler::ApiError,
    utils::probe::model::{ProbeRequest, ProviderMatch},
};

#[post("", wrap = "UserAuth::require(Permission::SubmissionReview)")]
pub async fn probe_file(
    req: web::Json<ProbeRequest>,
    drive_state: web::Data<Option<Arc<DriveState>>>,
) -> Result<HttpResponse, ApiError> {
    let result = ProviderMatch::from_url(req.url.as_str())
        .ok_or_else(|| ApiError::new(400, "Unsupported media URL"))?
        .resolve(drive_state.get_ref())
        .await?
        .probe()
        .await?;

    Ok(HttpResponse::Ok().json(result))
}

#[derive(OpenApi)]
#[openapi()]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/probe").service(probe_file));
}
