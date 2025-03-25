#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod db;
mod schema;
mod error_handler;
#[cfg(test)]
mod test_utils;

mod aredl;
mod custom_schema;
mod auth;
mod users;
mod roles;
mod clans;
mod docs;
mod page_helper;
mod cache_control;
mod refresh_leaderboard;
mod refresh_level_data;
mod clean_notifications;

use std::env;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::body::MessageBody;
use actix_web::Error;
use tracing_actix_web::{TracingLogger, RootSpanBuilder, DefaultRootSpanBuilder, root_span};
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing::Span;
use actix_cors::Cors;
use actix_web::{App, HttpServer, web};
use dotenv::dotenv;
use listenfd::ListenFd;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use std::fs;
use actix_web_prom::PrometheusMetricsBuilder;
use crate::docs::ApiDoc;
use crate::cache_control::CacheController;
use crate::refresh_leaderboard::start_leaderboard_refresher;
use crate::refresh_level_data::start_level_data_refresher;
use crate::clean_notifications::start_notifications_cleaner;


pub fn get_secret(var_name: &str) -> String {
    let value = env::var(var_name).expect(&format!("Please set {} in .env", var_name));
    if value.starts_with("/run/secrets/") {
        fs::read_to_string(value.trim()).expect("Failed to read secret file").trim().to_string()
    } else {
        value
    }
}

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=error".into())
        ))
        .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
        .json()
        .flatten_event(true) 
        .with_current_span(true)
        .with_span_list(false)
        .init();

    tracing::info!("Initializing...");

    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .build()
        .unwrap();

    let db_app_state = db::init_app_state();

    db_app_state.run_pending_migrations();

    start_leaderboard_refresher(db_app_state.clone());

    start_notifications_cleaner(db_app_state.clone());

    start_level_data_refresher(db_app_state.clone()).await;

    let auth_app_state = auth::init_app_state().await;
    

    let mut listenfd = ListenFd::from_env();
    let mut server = HttpServer::new(move || {

        let cors = Cors::permissive();

        let docs_html = "\
            <!doctype html><html><head><meta charset=\"utf-8\"><script type=\"module\" src=\"https://unpkg.com/rapidoc/dist/rapidoc-min.js\"></script></head><body><rapi-doc \
                spec-url = \"openapi.json\" \
                show-method-in-nav-bar = as-colored-block \
                render-style = focused \
                allow-spec-url-load = false \
                allow-spec-file-load = false \
                allow-spec-file-download = false \
                allow-server-selection = false \
                show-components = true \
                schema-description-expanded = true \
                persist-auth = true \
                default-schema-tab = schema \
                schema-expand-level = 1 \
                font-size = largest \
                bg-color = #1c1c1c \
                header-color = #ff6f00 \
                text-color =  #ffffff \
                primary-color = #ff6f00 \
                nav-bg-color = #424242 \
                nav-accent-color = #ff6f00 \
             >\
                <header style=\"color:white; font-weight: lighter; font-size: 1.5rem;\" slot=\"header\">All Rated Extreme Demons List | API v2 Documentation</header>\
                <img style=\"padding: 0.5rem; height: 3rem;\" slot=\"logo\"  src=\"https://aredl.net/logo.png\"/>
            </rapi-doc></body></html>";

        App::new()
            .wrap(prometheus.clone())
            .service(
                web::scope("/api")
                    .app_data(web::Data::new(auth_app_state.clone()))
                    .app_data(web::Data::new(db_app_state.clone()))
                    .wrap(CacheController::default_no_store())
                    .wrap(TracingLogger::<AppRootSpanBuilder>::new())
                    .wrap(cors)
                    .configure(aredl::init_routes)
                    .configure(auth::init_routes)
                    .configure(users::init_routes)
                    .configure(roles::init_routes)
                    .configure(clans::init_routes)
            )
            .service(
                RapiDoc::with_openapi("/openapi.json", ApiDoc::openapi()).path("/docs").custom_html(docs_html),
            )
    });

    server = match listenfd.take_tcp_listener(0)? {
        Some(listener) => server.listen(listener)?,
        None => {
            let host = get_secret("HOST");
            let port = get_secret("PORT");
            server.bind(format!("{}:{}", host, port))?
        }
    };

    server.run().await
}

pub struct AppRootSpanBuilder;

impl RootSpanBuilder for AppRootSpanBuilder {
    fn on_request_start(request: &ServiceRequest) -> Span {
        let query = request.query_string();
        let span = {
            root_span!(request, user_id = tracing::field::Empty, query = %query, body = tracing::field::Empty)
        };
        span
    }

    fn on_request_end<B: MessageBody>(span: Span, outcome: &Result<ServiceResponse<B>, Error>) {
        DefaultRootSpanBuilder::on_request_end(span, outcome);
    }
}