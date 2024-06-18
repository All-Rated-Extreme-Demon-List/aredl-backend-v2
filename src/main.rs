#[macro_use]
extern crate diesel;#[macro_use]
extern crate diesel_migrations;

mod db;
mod schema;
mod error_handler;

mod aredl;
mod custom_schema;
mod auth;
mod users;
mod page_helper;

use std::env;
use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use listenfd::ListenFd;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    db::init();

    let auth_app_state = auth::init_app_state().await;

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || App::new().service(
        web::scope("/api")
            .app_data(web::Data::new(auth_app_state.clone()))
            .configure(aredl::init_routes)
            .configure(auth::init_routes)
    ));

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
