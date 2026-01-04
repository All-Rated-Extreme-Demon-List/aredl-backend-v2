use actix_web::web;
use utoipa::OpenApi;

use crate::aredl::statistics::{records, submissions};

#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "AREDL - Statistics", description = "Endpoints for various statistics about the list")
    ),
	nest(
        (path = "/records", api=records::ApiDoc),
		(path = "/submissions", api=submissions::ApiDoc),
    )
)]
pub struct ApiDoc;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/statistics")
            .configure(records::init_routes)
            .configure(submissions::init_routes),
    );
}
