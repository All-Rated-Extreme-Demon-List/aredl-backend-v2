use std::rc::Rc;
use std::sync::Arc;
use std::task::{Context, Poll};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::{HttpMessage, web};
use actix_web::error::ErrorUnauthorized;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl};
use diesel::dsl::max;
use futures_util::future::{LocalBoxFuture, ready, Ready};
use uuid::Uuid;

use crate::auth::app_state::AuthAppState;
use crate::auth::token::{decode_token, UserClaims};
use crate::db;
use crate::error_handler::ApiError;
use crate::schema::{permissions, roles};
use crate::schema::user_roles;

pub struct UserAuth {
    required_perm: Option<String>,
}

impl UserAuth {
    pub fn load() -> Self {
        UserAuth { required_perm: None }
    }

    pub fn require(permission: &str) -> Self {
        UserAuth { required_perm: Some(permission.to_string()) }
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
    required_perm: Option<String>,
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
        let token = req
            .cookie("token")
            .map(|c| c.value().to_string())
            .or_else(|| {
                req.headers()
                    .get(openidconnect::http::header::AUTHORIZATION)
                    .map(|h| h.to_str().unwrap().split_at(7).1.to_string())
            });

        if token.is_none() {
            return if require_auth {
                Box::pin(ready(Err(ErrorUnauthorized(ApiError::new(403, "Unauthorized")))))
            } else {
                // auth is not required
                let fut = self.service.call(req);
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            }
        }

        let app_state = req.app_data::<web::Data<Arc<AuthAppState>>>().unwrap();

        let user_claims = match decode_token(
            &token.unwrap(),
            &app_state.jwt_decoding_key,
        ) {
            Ok(claims) => claims,
            Err(e) => return Box::pin(ready(Err(ErrorUnauthorized(ApiError::new(403, e.error_message.as_str())))))
        };

        let user_id = user_claims.user_id;

        req.extensions_mut().insert::<UserClaims>(user_claims);

        let fut = self.service.call(req);

        let required_permission = self.required_perm.clone().unwrap();

        Box::pin(async move {
            let has_permission = web::block(move || check_permission(user_id, required_permission.as_str())).await?
                .map_err(|_| ApiError::new(500, "Failed to retrieve permission"))?;

            if !has_permission {
                return Err(ErrorUnauthorized(ApiError::new(403, "Required permission to access this endpoint is missing")))
            }

            let res = fut.await?;
            Ok(res)
        })
    }
}

fn get_privilege_level(user_id: Uuid) -> Result<i32, ApiError> {
    let privilege_level: Option<i32> = user_roles::table
        .inner_join(roles::table.on(roles::id.eq(user_roles::role_id)))
        .filter(user_roles::user_id.eq(user_id))
        .select(max(roles::privilege_level))
        .first(&mut db::connection()?)
        .unwrap_or(None);
    Ok(privilege_level.unwrap_or(0))
}

fn check_permission(user_id: Uuid, permission: &str) -> Result<bool, ApiError> {
    let max_privilege = get_privilege_level(user_id)?;
    let required_privilege = permissions::table
        .filter(permissions::permission.eq(permission))
        .select(permissions::privilege_level)
        .first::<i32>(&mut db::connection()?)?;
    Ok(required_privilege <= max_privilege)
}