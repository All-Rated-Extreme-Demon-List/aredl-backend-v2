#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod db;
mod schema;
mod error_handler;

mod aredl;
mod custom_schema;
mod auth;
mod users;
mod page_helper;
mod cache_control;
mod refresh_leaderboard;
mod refresh_level_data;

use std::env;
use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use listenfd::ListenFd;
use crate::cache_control::CacheController;
use crate::refresh_leaderboard::start_leaderboard_refresher;
use crate::refresh_level_data::start_level_data_refresher;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let db_app_state = db::init_app_state();

    db_app_state.run_pending_migrations();

    start_leaderboard_refresher(db_app_state.clone());

    start_level_data_refresher(db_app_state.clone()).await;

    let auth_app_state = auth::init_app_state().await;

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {

        let cors = Cors::permissive();

        App::new()
            .service(
                web::scope("/api")
                    .app_data(web::Data::new(auth_app_state.clone()))
                    .app_data(web::Data::new(db_app_state.clone()))
                    .wrap(CacheController::default_no_store())
                    .wrap(cors)
                    .configure(aredl::init_routes)
                    .configure(auth::init_routes)
                    .configure(users::init_routes)
            )
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = env::var("HOST").expect("Please set host in .env");
            let port = env::var("PORT").expect("Please set port in .env");
            server.bind(format!("{}:{}", host, port))?
        }
    };

    server.run().await
}
