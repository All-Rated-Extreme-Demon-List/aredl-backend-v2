use actix_web::{post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

use crate::{
    auth::{Permission, UserAuth},
    drive::DriveState,
    error_handler::ApiError,
    utils::probe::model::{detect_provider, probe_url, resolve_media, ProbeRequest, ProbeResponse},
};

#[utoipa::path(
    post,
    summary = "[Staff] Probe Media URL",
    request_body = ProbeRequest,
    responses(
        (status = 200, body = ProbeResponse)
    ),
    security(
        ("access_token" = ["SubmissionReview"]),
        ("api_key" = ["SubmissionReview"]),
    )
)]
#[post("", wrap = "UserAuth::require(Permission::SubmissionReview)")]
pub async fn probe_file(
    req: web::Json<ProbeRequest>,
    drive_state: web::Data<Option<Arc<DriveState>>>,
) -> Result<HttpResponse, ApiError> {
    let matched = detect_provider(req.url.as_str())
        .ok_or_else(|| ApiError::new(400, "Unsupported media URL"))?;

    let resolved_media = resolve_media(&matched, drive_state.get_ref()).await?;

    let probe_result = probe_url(&resolved_media).await?;

    Ok(HttpResponse::Ok().json(probe_result))
}

#[derive(OpenApi)]
#[openapi(components(schemas(ProbeRequest, ProbeResponse)), paths(probe_file))]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/probe").service(probe_file));
}
