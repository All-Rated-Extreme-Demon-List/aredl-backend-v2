use actix_http::header;
use actix_web::body::BoxBody;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{web, Error, HttpMessage, HttpRequest, ResponseError};
use futures_util::future::{ready, LocalBoxFuture, Ready};
use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll};

use crate::app_data::auth::AuthAppState;
use crate::app_data::db::DbAppState;
use crate::auth::token::{decode_token, decode_user_claims, UserClaims};
use crate::auth::{permission, Permission};

use crate::auth::token::check_token_valid;
use crate::error_handler::ApiError;

pub struct UserAuth {
    required_perm: Option<Permission>,
}

impl UserAuth {
    pub fn load() -> Self {
        UserAuth {
            required_perm: None,
        }
    }

    pub fn require(permission: Permission) -> Self {
        UserAuth {
            required_perm: Some(permission),
        }
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

impl<S> AuthMiddleware<S> {
    fn error_future(
        http_req: HttpRequest,
        api_err: ApiError,
    ) -> LocalBoxFuture<'static, Result<ServiceResponse<BoxBody>, Error>> {
        let http_res = api_err.error_response().map_into_boxed_body();
        Box::pin(ready(Ok(ServiceResponse::new(http_req, http_res))))
    }
}

impl<S> Service<ServiceRequest> for AuthMiddleware<S>
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
        let require_auth = self.required_perm.clone().is_some();
        let (http_req, payload) = req.into_parts();
        let token = http_req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok())
            .and_then(|value| value.strip_prefix("Bearer "))
            .filter(|token| !token.is_empty())
            .map(str::to_owned);

        let Some(token) = token else {
            return if require_auth {
                Self::error_future(
                    http_req,
                    ApiError::Unauthorized("You must be authenticated to access this endpoint"),
                )
            } else {
                // auth is not required
                let fut = self
                    .service
                    .call(ServiceRequest::from_parts(http_req, payload));
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            };
        };

        let Some(app_state) = http_req.app_data::<web::Data<Arc<AuthAppState>>>().cloned() else {
            return Self::error_future(
                http_req,
                ApiError::InternalServerError("Authentication unavailable"),
            );
        };

        let Some(db_state) = http_req.app_data::<web::Data<Arc<DbAppState>>>().cloned() else {
            return Self::error_future(
                http_req,
                ApiError::InternalServerError("Database unavailable"),
            );
        };

        let token_claims =
            match decode_token(token, &app_state.jwt_decoding_key, &["initial", "access"]) {
                Ok(claims) => claims,
                Err(_) => {
                    return Self::error_future(
                        http_req,
                        ApiError::Unauthorized("Failed to decode token"),
                    )
                }
            };

        let user_claims = match decode_user_claims(&token_claims) {
            Ok(claims) => claims,
            Err(_) => {
                return Self::error_future(
                    http_req,
                    ApiError::Unauthorized("Failed to extract user claims"),
                )
            }
        };

        let mut conn = match db_state.connection() {
            Ok(conn) => conn,
            Err(error) => return Self::error_future(http_req, error),
        };

        if check_token_valid(&token_claims, &user_claims, &mut conn).is_err() {
            return Self::error_future(
                http_req,
                ApiError::Unauthorized("Token has been invalidated"),
            );
        }

        let user_id = user_claims.user_id;

        tracing::Span::current().record("user_id", tracing::field::display(user_id));

        if let Some(required_perm) = self.required_perm.clone() {
            let has_permission =
                permission::check_user_permission(&mut conn, user_id, required_perm.clone());
            match has_permission {
                Ok(permission) => {
                    if !permission {
                        return Self::error_future(
                            http_req,
                            ApiError::Forbidden(
                                format!("You do not have the required permission ({}) to access this endpoint",required_perm)
                                .as_str(),
                            ),
                        );
                    }
                }
                Err(_) => {
                    return Self::error_future(
                        http_req,
                        ApiError::InternalServerError("Failed to load permissions"),
                    )
                }
            }
        }

        http_req.extensions_mut().insert::<UserClaims>(user_claims);

        let fut = self
            .service
            .call(ServiceRequest::from_parts(http_req, payload));

        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
