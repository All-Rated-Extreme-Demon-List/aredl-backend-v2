#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_migrations;

mod app_data;
mod error_handler;
mod schema;

#[cfg(test)]
mod tests;

#[cfg(test)]
mod test_utils;

mod aredl;
mod arepl;
mod auth;
mod cache_control;
mod clans;
mod docs;
mod health;
mod notifications;
mod page_helper;
mod roles;
mod scheduled;
mod shifts;
mod users;
mod utils;

use crate::app_data::{auth as auth_data, db, drive};
use crate::cache_control::CacheController;
use crate::docs::ApiDoc;
use crate::scheduled::{
    data_cleaner::start_data_cleaner, refresh_discord_avatars::start_discord_avatars_refresher,
    refresh_level_data::start_level_data_refresher, refresh_matviews::start_matviews_refresher,
    shifts_creator::start_recurrent_shift_creator,
};
use actix_cors::Cors;
use actix_governor::GovernorConfigBuilder;
use actix_http::StatusCode;
use actix_web::body::MessageBody;
use actix_web::dev::{ServiceRequest, ServiceResponse};
use actix_web::middleware::NormalizePath;
use actix_web::Error;
use actix_web::{web, App, HttpServer};
use actix_web_prom::PrometheusMetricsBuilder;

use dotenv::dotenv;
use listenfd::ListenFd;
use notifications::WebsocketNotification;
use std::env;
use std::fs;
use tokio::sync::broadcast;
use tracing::Span;
use tracing_actix_web::{root_span, DefaultRootSpanBuilder, RootSpanBuilder, TracingLogger};
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::EnvFilter;
use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();

    if cfg!(debug_assertions) {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=error".into()),
            ))
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .pretty()
            .init();
    } else {
        tracing_subscriber::fmt()
            .with_env_filter(EnvFilter::new(
                std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tower_http=error".into()),
            ))
            .with_span_events(FmtSpan::NEW | FmtSpan::CLOSE)
            .json()
            .flatten_event(true)
            .with_current_span(true)
            .with_span_list(false)
            .init();
    }

    tracing::info!("Initializing...");

    let (notify_tx, _notify_rx) = broadcast::channel::<WebsocketNotification>(100);

    let prometheus = PrometheusMetricsBuilder::new("api")
        .endpoint("/metrics")
        .exclude_status(StatusCode::NOT_FOUND)
        .mask_unmatched_patterns("/unmatched")
        .build()
        .unwrap();

    let db_app_state = db::init_app_state();

    let auth_app_state = auth_data::init_app_state().await;

    let drive_app_state = drive::init_app_state();

    db_app_state.run_pending_migrations();

    start_matviews_refresher(db_app_state.clone()).await;

    start_data_cleaner(db_app_state.clone(), notify_tx.clone()).await;

    start_level_data_refresher(db_app_state.clone()).await;

    start_recurrent_shift_creator(db_app_state.clone(), notify_tx.clone()).await;

    start_discord_avatars_refresher(db_app_state.clone()).await;

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
                <img style=\"padding: 0.5rem; height: 3rem;\" slot=\"logo\"  src=\"https://aredl.net/assets/logo.webp\"/>
            </rapi-doc></body></html>";

        let _governor_conf = GovernorConfigBuilder::default()
            .requests_per_minute(100)
            .burst_size(20)
            .use_headers()
            .finish()
            .expect("invalid governor config");

        App::new()
            .wrap(prometheus.clone())
            .service(
                web::scope("/api")
                    .app_data(web::Data::new(auth_app_state.clone()))
                    .app_data(web::Data::new(db_app_state.clone()))
                    .app_data(web::Data::new(drive_app_state.clone()))
                    .app_data(web::Data::new(notify_tx.clone()))
                    .wrap(CacheController::default_no_store())
                    .wrap(NormalizePath::trim())
                    .wrap(TracingLogger::<AppRootSpanBuilder>::new())
                    .wrap(cors)
                    .configure(aredl::init_routes)
                    .configure(arepl::init_routes)
                    .configure(auth::init_routes)
                    .configure(users::init_routes)
                    .configure(roles::init_routes)
                    .configure(clans::init_routes)
                    .configure(notifications::init_routes)
                    .configure(health::init_routes)
                    .configure(shifts::init_routes)
                    .configure(utils::init_routes),
            )
            .service(
                RapiDoc::with_openapi("/openapi.json", ApiDoc::openapi())
                    .path("/docs")
                    .custom_html(docs_html),
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

pub fn get_secret(var_name: &str) -> String {
    let value = env::var(var_name).expect(&format!("Please set {} in .env", var_name));
    if value.starts_with("/run/secrets/") {
        fs::read_to_string(value.trim())
            .expect("Failed to read secret file")
            .trim()
            .to_string()
    } else {
        value
    }
}

pub fn get_optional_secret(var_name: &str) -> Option<String> {
    let value = match env::var(var_name) {
        Ok(v) => v,
        Err(_) => {
            tracing::warn!("Optional .env {} not set", var_name);
            return None;
        }
    };
    if value.starts_with("/run/secrets/") {
        match fs::read_to_string(value.trim()) {
            Ok(v) => Some(v.trim().to_string()),
            Err(e) => {
                tracing::warn!("Failed to read secret file for {}: {}", var_name, e);
                return None;
            }
        }
    } else {
        Some(value)
    }
}
