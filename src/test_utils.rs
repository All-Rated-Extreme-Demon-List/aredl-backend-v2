use crate::app_data::{
    auth::{init_app_state as auth_init_app_state, AuthAppState},
    db::DbAppState,
    providers::init_app_state as providers_init_app_state,
};
use actix_http::Request;

use actix_web::{
    body::BoxBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use actix_web::{test, web::Data, App};

use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::sync::Arc;
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

#[cfg(test)]
pub async fn init_test_app() -> (
    impl Service<Request, Response = ServiceResponse<BoxBody>, Error = Error>,
    Arc<DbAppState>,
    Arc<AuthAppState>,
    tokio::sync::broadcast::Sender<crate::notifications::WebsocketNotification>,
) {
    use actix_web::middleware::NormalizePath;
    use tokio::sync::broadcast;
    use tracing_actix_web::TracingLogger;

    use crate::{
        app_data::db::init_test_db_state, notifications::WebsocketNotification, AppRootSpanBuilder,
    };

    dotenv::dotenv().ok();

    let auth_app_state = auth_init_app_state().await;

    let (notify_tx, _notify_rx) = broadcast::channel::<WebsocketNotification>(100);

    let db_app_state = init_test_db_state();

    let providers_app_state = providers_init_app_state().await;

    let app = test::init_service(
        App::new()
            .app_data(Data::new(db_app_state.clone()))
            .app_data(Data::new(auth_app_state.clone()))
            .app_data(Data::new(notify_tx.clone()))
            .app_data(Data::new(providers_app_state.clone()))
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
            .configure(crate::shifts::init_routes)
            .configure(crate::health::init_routes),
    )
    .await;

    (app, db_app_state, auth_app_state, notify_tx)
}
