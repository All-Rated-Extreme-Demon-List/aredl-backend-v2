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

use std::env;
use actix_cors::Cors;
use actix_session::config::{CookieContentSecurity, PersistentSession};
use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;
use actix_web::{App, HttpServer, web};
use actix_web::cookie::{Key, SameSite};
use dotenv::dotenv;
use listenfd::ListenFd;
use crate::cache_control::CacheController;
use crate::refresh_leaderboard::start_leaderboard_refresher;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    let db_app_state = db::init_app_state();

    db_app_state.run_pending_migrations();

    start_leaderboard_refresher(db_app_state.clone());

    let auth_app_state = auth::init_app_state().await;

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {
        let cookie_key = Key::from(
            env::var("COOKIE_KEY")
                .expect("COOKIE_KEY not set").as_bytes());

        let cors = Cors::permissive();

        App::new()
            .service(
                web::scope("/api")
                    .app_data(web::Data::new(auth_app_state.clone()))
                    .app_data(web::Data::new(db_app_state.clone()))
                    .wrap(CacheController::default_no_store())
                    .wrap(SessionMiddleware::builder(CookieSessionStore::default(), cookie_key)
                        .session_lifecycle(PersistentSession::default())
                        .cookie_secure(true)
                        .cookie_same_site(SameSite::None)
                        .cookie_content_security(CookieContentSecurity::Private)
                        .build()
                    )
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
