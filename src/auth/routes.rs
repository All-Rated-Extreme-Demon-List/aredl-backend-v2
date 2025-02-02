use actix_web::web;
use utoipa::OpenApi;
use crate::auth::discord;
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Authentication", description = "Authentication related endpoints")
    ),
    nest(
        (path = "/discord", api = discord::ApiDoc )
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.configure(discord::init_discord_routes);
}