use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{HttpMessage, web, HttpResponse};
use futures_util::future::{LocalBoxFuture, ready, Ready};

use crate::auth::app_state::AuthAppState;
use crate::auth::{Permission, permission};
use crate::auth::token::{decode_token, decode_user_claims, UserClaims};
use crate::db::DbAppState;

use crate::auth::token::check_token_valid;

pub struct UserAuth {
    required_perm: Option<Permission>,
}

impl UserAuth {
    pub fn load() -> Self {
        UserAuth { required_perm: None }
    }

    pub fn require(permission: Permission) -> Self {
        UserAuth { required_perm: Some(permission) }
    }
}

impl<S> Transform<S, ServiceRequest> for UserAuth
    where
        S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error,
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Transform = AuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddleware {
            service: Rc::new(service),
            required_perm: self.required_perm.clone(),
        }))
    }
}

pub struct AuthMiddleware<S> {
    service: Rc<S>,
    required_perm: Option<Permission>,
}

impl<S> Service<ServiceRequest> for AuthMiddleware<S>
    where
        S: Service<
            ServiceRequest,
            Response = ServiceResponse<actix_web::body::BoxBody>,
            Error = actix_web::Error
        > + 'static,
{
    type Response = ServiceResponse<actix_web::body::BoxBody>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, actix_web::Error>>;

    fn poll_ready(&self, ctx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let require_auth = self.required_perm.clone().is_some();
        let (http_req, payload) = req.into_parts();
        let token = http_req.headers()
            .get(openidconnect::http::header::AUTHORIZATION)
            .map(|h| h.to_str().unwrap().split_at(7).1.to_string());

        if token.is_none() {
            return if require_auth {
                Box::pin(ready(
                    Ok(ServiceResponse::new(http_req, HttpResponse::Forbidden().reason("Unauthorized").finish()))
                ))
            } else {
                // auth is not required
                let fut = self.service.call(ServiceRequest::from_parts(http_req, payload));
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            }
        }

        let app_state = http_req
            .app_data::<web::Data<Arc<AuthAppState>>>()
            .unwrap();

        let db_state = http_req
            .app_data::<web::Data<Arc<DbAppState>>>()
            .unwrap().clone();

        let token_claims = match decode_token(
            &token.unwrap(),
            &app_state.jwt_decoding_key,
            &["initial", "access"],
        ) {
            Ok(claims) => claims,
            Err(_) => {
                return Box::pin(ready(
                    Ok(ServiceResponse::new(http_req, HttpResponse::Forbidden().reason("Failed to decode token").finish()))
                ))
            }

          };

        let user_claims = match decode_user_claims(&token_claims) {
            Ok(claims) => claims,
            Err(_) => {
                return Box::pin(ready(
                    Ok(ServiceResponse::new(http_req, HttpResponse::Forbidden().reason("Failed to extract user claims").finish()))
                ))
            }
        };

        let mut conn = &mut db_state.connection().unwrap();

        if let Err(_) = check_token_valid(&token_claims, &user_claims, &mut conn) {
            return Box::pin(ready(
                Ok(ServiceResponse::new(http_req, HttpResponse::Forbidden().reason("Token has been invalidated").finish()))
            ))
        }

        let user_id = user_claims.user_id;

        tracing::Span::current().record("user_id", &tracing::field::display(user_id));

        match self.required_perm.clone() {
            Some(required_perm) => {
                let has_permission = permission::check_permission(db_state, user_id, required_perm);
                match has_permission {
                    Ok(permission) => {
                        if !permission {
                            return Box::pin(ready(
                                Ok(ServiceResponse::new(http_req, HttpResponse::Forbidden().reason("No permission to access this endpoint").finish()))
                            ))
                        }
                    },
                    Err(_) => {
                        return Box::pin(ready(
                            Ok(ServiceResponse::new(http_req, HttpResponse::Forbidden().reason("Failed to load permissions").finish()))
                        ))
                    }
                }
            },
            None => {}
        }

        http_req.extensions_mut().insert::<UserClaims>(user_claims);

        let fut = self.service.call(ServiceRequest::from_parts(http_req, payload));

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}