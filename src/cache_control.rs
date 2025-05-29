use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{self, CacheControl, CacheDirective, TryIntoHeaderValue},
    Error,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::{
    marker::PhantomData,
    rc::Rc,
    task::{Context, Poll},
};

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

impl<S, B> Transform<S, ServiceRequest> for CacheController
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = CacheControlMiddleware<S, B>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        let header_value = CacheControl(self.cache_directive.clone())
            .try_into_value()
            .expect("Invalid cache directive");

        ready(Ok(CacheControlMiddleware::<S, B> {
            service: Rc::new(service),
            header_value,
            replace: self.replace,
            _marker: PhantomData,
        }))
    }
}

pub struct CacheControlMiddleware<S, B> {
    service: Rc<S>,
    header_value: header::HeaderValue,
    replace: bool,
    _marker: PhantomData<B>,
}

impl<S, B> Service<ServiceRequest> for CacheControlMiddleware<S, B>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let fut = self.service.call(req);
        let replace = self.replace;
        let header_value = self.header_value.clone();

        Box::pin(async move {
            let mut res = fut.await?;
            if !res.headers().contains_key(header::CACHE_CONTROL) || replace {
                res.headers_mut()
                    .insert(header::CACHE_CONTROL, header_value);
            }
            Ok(res)
        })
    }
}
