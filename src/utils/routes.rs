use actix_web::web;
use utoipa::OpenApi;

use crate::utils::probe;

#[derive(OpenApi)]
#[openapi(
    nest(
        (path = "/probe", api = probe::ApiDoc),
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(web::scope("/utils").configure(probe::init_routes));
}
