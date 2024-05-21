use actix_web::web;
use crate::auth::discord::init_discord_routes;

pub fn init_routes(config: &mut web::ServiceConfig) {
    config.configure(init_discord_routes);
}