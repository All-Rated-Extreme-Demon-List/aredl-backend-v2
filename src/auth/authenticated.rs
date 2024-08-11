use std::future::{Ready, ready};
use std::sync::Arc;
use actix_web::{FromRequest, HttpMessage, HttpRequest, web};
use actix_web::dev::Payload;
use serde::{Deserialize, Serialize};
use crate::auth::{Permission, permission};
use crate::auth::token::UserClaims;
use crate::db::DbAppState;
use crate::error_handler::ApiError;

#[derive(Serialize, Deserialize, Debug)]
pub struct Authenticated(UserClaims);

impl FromRequest for Authenticated {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let value = req.extensions().get::<UserClaims>().cloned();

        let result = match value {
            Some(claims) => Ok(Authenticated(claims)),
            None => Err(ApiError::new(401, "Authentication error")),
        };

        ready(result)
    }
}

impl Authenticated {
    pub fn has_permission(&self, db: web::Data<Arc<DbAppState>>, permission: Permission) -> Result<bool, ApiError> {
        permission::check_permission(db, self.user_id, permission)
    }
}

impl std::ops::Deref for Authenticated {
    type Target = UserClaims;

    /// Implement the deref method to access the inner User value of Authenticated.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}