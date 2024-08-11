use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll};
use actix_session::SessionExt;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{HttpMessage, web};
use actix_web::error::ErrorUnauthorized;
use futures_util::future::{LocalBoxFuture, ready, Ready};

use crate::auth::app_state::AuthAppState;
use crate::auth::{Permission, permission};
use crate::auth::token::{decode_token, UserClaims};
use crate::db::DbAppState;
use crate::error_handler::ApiError;

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
        let require_auth = self.required_perm.is_some();
        let token = req.get_session().get::<String>("token")
            .unwrap_or(None)
            .or_else(|| {
                req.headers()
                    .get(openidconnect::http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        if token.is_none() {
            return if require_auth {
                Box::pin(ready(
                    Err(ErrorUnauthorized(
                        ApiError::new(401, "Unauthorized")
                    ))))
            } else {
                // auth is not required
                let fut = self.service.call(req);
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            }
        }

        let app_state = req
            .app_data::<web::Data<Arc<AuthAppState>>>()
            .unwrap();

        let db_state = req
            .app_data::<web::Data<Arc<DbAppState>>>()
            .unwrap().clone();

        let user_claims = match decode_token(
            &token.unwrap(),
            &app_state.jwt_decoding_key,
        ) {
            Ok(claims) => claims,
            Err(e) =>
                return Box::pin(ready(
                    Err(ErrorUnauthorized(ApiError::new(403, e.error_message.as_str())))
                ))
        };

        let user_id = user_claims.user_id;

        req.extensions_mut().insert::<UserClaims>(user_claims);

        let fut = self.service.call(req);

        if !require_auth {
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            })
        }

        let required_permission = self.required_perm.clone().unwrap();

        Box::pin(async move {
            let has_permission = web::block(move ||
                permission::check_permission(db_state, user_id, required_permission)
            ).await?
                .map_err(|_| ApiError::new(500, "Failed to retrieve permission"))?;

            if !has_permission {
                return Err(ErrorUnauthorized(
                    ApiError::new(403, "Required permission to access this endpoint is missing"))
                )
            }

            let res = fut.await?;
            Ok(res)
        })
    }
}