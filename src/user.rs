use actix_web::{FromRequest, HttpRequest};
use actix_web::dev::Payload;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Queryable)]
pub struct User {
    pub id: Uuid,
    pub user_name: String,
    pub global_name: String,
    pub placeholder: bool,
}

impl FromRequest for User {
    type Error = actix_web::Error;
    type Future = std::pin::Pin<Box<dyn std::future::Future<Output = Result<Self, Self::Error>>>>;

    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        todo!()
    }
}
