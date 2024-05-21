use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error_handler::ApiError;

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenData {
    pub user_id: Uuid,
    pub is_api_key: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: TokenData,
    pub iat: usize,
    pub exp: usize,
}

pub fn create_token(
    token_data: TokenData,
    encoding_key: &EncodingKey,
    expires_in_seconds: i64,
) -> Result<String, ApiError> {

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::minutes(expires_in_seconds)).timestamp() as usize;
    let claims = TokenClaims {
        sub: token_data,
        exp,
        iat,
    };

    encode(
        &Header::default(),
        &claims,
        &encoding_key,
    ).map_err(|_| ApiError::new(400, "Failed to create token!"))
}

pub fn decode_token<T: Into<String>>(token: T, decoding_key: &DecodingKey) -> Result<TokenData, ApiError> {
    let decoded = decode::<TokenClaims>(
        &token.into(),
        &decoding_key,
        &Validation::new(Algorithm::HS256),
    );
    match decoded {
        Ok(token) => Ok(token.claims.sub),
        Err(_) => Err(ApiError::new(401, "Invalid token!")),
    }
}