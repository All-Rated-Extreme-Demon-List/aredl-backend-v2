use actix_web::web;
use utoipa::OpenApi;
use crate::auth::{discord, apikey, logout};
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Authentication", description = "Authentication related endpoints")
    ),
    nest(
        (path = "/discord", api = discord::ApiDoc ),
        (path = "/api-key", api = apikey::ApiDoc ),
        (path = "/logout-all", api = logout::ApiDoc)
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.configure(discord::init_routes);
    config.configure(apikey::init_routes);
    config.configure(logout::init_routes);
}