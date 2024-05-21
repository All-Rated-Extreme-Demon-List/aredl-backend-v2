use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error_handler::ApiError;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: Uuid,
    pub iat: usize,
    pub exp: usize,
}

pub fn create_token(
    user_id: Uuid,
    secret: &[u8],
    expires_in_seconds: i64,
) -> Result<String, ApiError> {

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(expires_in_seconds)).timestamp() as usize;
    let claims = TokenClaims {
        sub: user_id,
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret),
    ).map_err(|_| ApiError::new(400, "Failed to create token!"))
}

pub fn decode_token<T: Into<String>>(token: T, secret: &[u8]) -> Result<Uuid, ApiError> {
    let decoded = decode::<TokenClaims>(
        &token.into(),
        &DecodingKey::from_secret(secret),
        &Validation::new(Algorithm::HS256),
    );
    match decoded {
        Ok(token) => Ok(token.claims.sub),
        Err(_) => Err(ApiError::new(401, "Invalid token!")),
    }
}