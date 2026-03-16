use actix_web::{
    body::MessageBody,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header::{self, CacheControl, CacheDirective, HeaderValue, TryIntoHeaderValue},
    Error,
};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::{
    marker::PhantomData,
    rc::Rc,
    task::{Context, Poll},
};

pub struct CacheController {
    cache_mode: CacheMode,
    replace: bool,
}

enum CacheMode {
    Static(Vec<CacheDirective>),
    AuthPublicMaxAge(u32),
}

impl CacheController {
    pub fn default_no_store() -> Self {
        Self {
            cache_mode: CacheMode::Static(vec![CacheDirective::NoCache, CacheDirective::NoStore]),
            replace: false,
        }
    }

    pub fn public_with_max_age(seconds: u32) -> Self {
        Self {
            cache_mode: CacheMode::Static(vec![
                CacheDirective::Public,
                CacheDirective::MaxAge(seconds),
            ]),
            replace: true,
        }
    }

    pub fn private_with_max_age(seconds: u32) -> Self {
        Self {
            cache_mode: CacheMode::Static(vec![
                CacheDirective::Private,
                CacheDirective::MaxAge(seconds),
            ]),
            replace: true,
        }
    }

    pub fn auth_public_with_max_age(seconds: u32) -> Self {
        Self {
            cache_mode: CacheMode::AuthPublicMaxAge(seconds),
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
        let cache_mode = match &self.cache_mode {
            CacheMode::Static(directives) => CacheModeRuntime::Static(
                CacheControl(directives.clone())
                    .try_into_value()
                    .expect("Invalid cache directive"),
            ),
            CacheMode::AuthPublicMaxAge(seconds) => CacheModeRuntime::AuthPublicMaxAge {
                public_header_value: CacheControl(vec![
                    CacheDirective::Public,
                    CacheDirective::MaxAge(*seconds),
                ])
                .try_into_value()
                .expect("Invalid cache directive"),
                private_header_value: CacheControl(vec![
                    CacheDirective::Private,
                    CacheDirective::MaxAge(*seconds),
                ])
                .try_into_value()
                .expect("Invalid cache directive"),
            },
        };

        ready(Ok(CacheControlMiddleware::<S, B> {
            service: Rc::new(service),
            cache_mode,
            replace: self.replace,
            _marker: PhantomData,
        }))
    }
}

enum CacheModeRuntime {
    Static(HeaderValue),
    AuthPublicMaxAge {
        public_header_value: HeaderValue,
        private_header_value: HeaderValue,
    },
}

pub struct CacheControlMiddleware<S, B> {
    service: Rc<S>,
    cache_mode: CacheModeRuntime,
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
        let has_authorization = req.headers().contains_key(header::AUTHORIZATION);
        let fut = self.service.call(req);
        let replace = self.replace;
        let header_value = match &self.cache_mode {
            CacheModeRuntime::Static(header_value) => header_value.clone(),
            CacheModeRuntime::AuthPublicMaxAge {
                public_header_value,
                private_header_value,
            } => {
                if has_authorization {
                    private_header_value.clone()
                } else {
                    public_header_value.clone()
                }
            }
        };
        let vary_on_authorization =
            matches!(self.cache_mode, CacheModeRuntime::AuthPublicMaxAge { .. });

        Box::pin(async move {
            let mut res = fut.await?;
            if !res.headers().contains_key(header::CACHE_CONTROL) || replace {
                res.headers_mut()
                    .insert(header::CACHE_CONTROL, header_value);
            }
            if vary_on_authorization {
                res.headers_mut()
                    .insert(header::VARY, HeaderValue::from_static("Authorization"));
            }
            Ok(res)
        })
    }
}
