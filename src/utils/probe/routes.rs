use actix_web::{post, web, HttpResponse};
use std::sync::Arc;
use utoipa::OpenApi;

use crate::{
    auth::{Permission, UserAuth},
    error_handler::ApiError,
    providers::ProvidersAppState,
    utils::probe::model::ProbeRequest,
};

#[post("", wrap = "UserAuth::require(Permission::SubmissionReviewFull)")]
pub async fn probe_file(
    req: web::Json<ProbeRequest>,
    providers_state: web::Data<Arc<ProvidersAppState>>,
) -> Result<HttpResponse, ApiError> {
    let providers_state = providers_state.get_ref();

    let matched = providers_state.parse_url(req.url.as_str())?;

    let result = providers_state
        .get_content_location(&matched)
        .await?
        .ok_or_else(|| {
            ApiError::UnprocessableEntity(
                "Not supported for this provider yet, or failed to retrieve content location",
            )
        })?
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
