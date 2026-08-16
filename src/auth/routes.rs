use crate::auth::{apikey, connected_accounts, discord, logout, patreon, refresh};
use actix_web::web;
use utoipa::OpenApi;
#[derive(OpenApi)]
#[openapi(
    tags(
        (name = "Authentication", description = "Authentication related endpoints")
    ),
    nest(
        (path = "/connected-accounts", api = connected_accounts::ApiDoc ),
        (path = "/discord", api = discord::ApiDoc ),
        (path = "/patreon", api = patreon::ApiDoc ),
        (path = "/refresh", api = refresh::ApiDoc ),
        (path = "/api-key", api = apikey::ApiDoc ),
        (path = "/logout-all", api = logout::ApiDoc)
    )
)]
pub struct ApiDoc;
pub fn init_routes(config: &mut web::ServiceConfig) {
    config.service(
        web::scope("/auth")
            .configure(connected_accounts::init_routes)
            .configure(discord::init_routes)
            .configure(patreon::init_routes)
            .configure(apikey::init_routes)
            .configure(logout::init_routes)
            .configure(refresh::init_routes),
    );
}
