use std::future::{Ready, ready};
use actix_web::{FromRequest, HttpMessage, HttpRequest};
use actix_web::dev::Payload;
use crate::auth::token::UserClaims;
use crate::error_handler::ApiError;

pub struct Authenticated(UserClaims);

impl FromRequest for Authenticated {
    type Error = ApiError;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _payload: &mut Payload) -> Self::Future {
        let value = req.extensions().get::<UserClaims>().cloned();

        let result = match value {
            Some(claims) => Ok(Authenticated(claims)),
            None => Err(ApiError::new(500, "Authentication error")),
        };

        ready(result)
    }
}

impl std::ops::Deref for Authenticated {
    type Target = UserClaims;

    /// Implement the deref method to access the inner User value of Authenticated.
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}