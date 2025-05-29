use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header;
use actix_web::http::header::{CacheControl, CacheDirective, TryIntoHeaderValue};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use openidconnect::http::HeaderValue;
use std::rc::Rc;
use std::task::{Context, Poll};

pub struct CacheController {
    cache_directive: Vec<CacheDirective>,
    replace: bool,
}

impl CacheController {
    pub fn default_no_store() -> Self {
        Self {
            cache_directive: vec![CacheDirective::NoCache, CacheDirective::NoStore],
            replace: false,
        }
    }

    pub fn public_with_max_age(seconds: u32) -> Self {
        Self {
            cache_directive: vec![CacheDirective::Public, CacheDirective::MaxAge(seconds)],
            replace: true,
        }
    }

    pub fn private_with_max_age(seconds: u32) -> Self {
        Self {
            cache_directive: vec![CacheDirective::Private, CacheDirective::MaxAge(seconds)],
            replace: true,
        }
    }
}

impl<S> Transform<S, ServiceRequest> for CacheController
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Transform = CacheControlMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(CacheControlMiddleware {
            service: Rc::new(service),
            cache_directive: CacheControl(self.cache_directive.clone())
                .try_into_value()
                .unwrap(),
            replace: self.replace,
        }))
    }
}

pub struct CacheControlMiddleware<S> {
    service: Rc<S>,
    cache_directive: HeaderValue,
    replace: bool,
}

impl<S> Service<ServiceRequest> for CacheControlMiddleware<S>
where
    S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, actix_web::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);

        let replace = self.replace;
        let cache_control = self.cache_directive.clone();
        Box::pin(async move {
            let mut res = fut.await?;
            if !res.headers().contains_key(header::CACHE_CONTROL) || replace {
                res.headers_mut()
                    .insert(header::CACHE_CONTROL, cache_control);
            }
            Ok(res)
        })
    }
}
