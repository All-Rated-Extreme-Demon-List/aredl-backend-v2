use crate::auth::{init_app_state, AuthAppState};
use crate::db::{DbAppState, DbConnection};
use crate::schema::permissions;
use actix_http::Request;

use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use actix_web::{test, web::Data, App};
use diesel::r2d2::{self, ConnectionManager};
use diesel::{ExpressionMethods, PgConnection, RunQueryDsl};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::sync::Arc;

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub struct BoxResponse;
impl<S, B> Transform<S, ServiceRequest> for BoxResponse
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = BoxResponseMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(BoxResponseMiddleware { service }))
    }
}

pub struct BoxResponseMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for BoxResponseMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: actix_web::body::MessageBody + 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res.map_into_boxed_body())
        })
    }
}

pub fn init_test_db_state() -> Arc<DbAppState> {
    let test_db_url = std::env::var("TEST_DATABASE_URL")
        .expect("TEST_DATABASE_URL must be set for running tests");

    let manager = ConnectionManager::<PgConnection>::new(test_db_url.clone());
    let pool = r2d2::Pool::builder()
        .test_on_check_out(true)
        .build(manager)
        .expect("Failed to create test database pool");

    let test_db_state = Arc::new(DbAppState { pool });

    test_db_state
        .connection()
        .unwrap()
        .revert_all_migrations(MIGRATIONS)
        .expect("Failed to revert test migrations");

    test_db_state.run_pending_migrations();

    let mut conn = test_db_state.connection().unwrap();

    let permissions_data = vec![
        ("plus", 5),
        ("submission_review", 15),
        ("record_modify", 20),
        ("placeholder_create", 25),
        ("user_modify", 25),
        ("pack_tier_modify", 30),
        ("pack_modify", 40),
        ("level_modify", 50),
        ("merge_review", 60),
        ("clan_modify", 70),
        ("notifications_subscribe", 75),
        ("user_ban", 85),
        ("direct_merge", 90),
        ("shift_manage", 95),
        ("role_manage", 100),
    ];

    diesel::insert_into(permissions::table)
        .values(
            permissions_data
                .iter()
                .map(|(permission, privilege_level)| {
                    (
                        permissions::permission.eq(*permission),
                        permissions::privilege_level.eq(*privilege_level),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .execute(&mut conn)
        .expect("Failed to insert permissions");

    test_db_state
}

#[cfg(test)]
pub async fn init_test_app() -> (
    impl Service<Request, Response = ServiceResponse<BoxBody>, Error = Error>,
    DbConnection,
    Arc<AuthAppState>,
) {
    use actix_web::middleware::NormalizePath;
    use tokio::sync::broadcast;
    use tracing_actix_web::TracingLogger;

    use crate::{notifications::WebsocketNotification, AppRootSpanBuilder};

    dotenv::dotenv().ok();

    let auth_app_state = init_app_state().await;

    let (notify_tx, _notify_rx) = broadcast::channel::<WebsocketNotification>(100);

    let db_app_state = init_test_db_state();
    let conn = db_app_state.connection().unwrap();

    let app = test::init_service(
        App::new()
            .app_data(Data::new(db_app_state))
            .app_data(Data::new(auth_app_state.clone()))
            .app_data(Data::new(notify_tx.clone()))
            .wrap(NormalizePath::trim())
            .wrap(TracingLogger::<AppRootSpanBuilder>::new())
            .wrap(BoxResponse)
            .configure(crate::users::init_routes)
            .configure(crate::aredl::init_routes)
            .configure(crate::arepl::init_routes)
            .configure(crate::auth::init_routes)
            .configure(crate::roles::init_routes)
            .configure(crate::clans::init_routes)
            .configure(crate::notifications::init_routes)
            .configure(crate::health::init_routes),
    )
    .await;

    (app, conn, auth_app_state)
}
