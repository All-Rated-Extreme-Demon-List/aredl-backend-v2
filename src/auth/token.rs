use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error_handler::ApiError;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct UserClaims {
    pub user_id: Uuid,
    pub is_api_key: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn create_token(
    user_claims: UserClaims,
    encoding_key: &EncodingKey,
    expires_in: Duration,
) -> Result<(String, DateTime<Utc>), ApiError> {

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let expire_datetime = now + expires_in;
    let exp = expire_datetime.timestamp() as usize;
    let user_claims_serialized = serde_json::to_string(&user_claims)
        .map_err(|_| ApiError::new(500, "Failed to serialize user claims!"))?;

    let claims = TokenClaims {
        sub: user_claims_serialized,
        exp,
        iat,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &encoding_key,
    ).map_err(|_| ApiError::new(400, "Failed to create token!"))?;

    Ok((token, expire_datetime))
}

pub fn decode_token<T: Into<String>>(token: T, decoding_key: &DecodingKey) -> Result<UserClaims, ApiError> {
    let token_str = token.into();

    let decoded = decode::<TokenClaims>(
        &token_str,
        &decoding_key,
        &Validation::new(Algorithm::HS256),
    ).map_err(|e| ApiError::new(401, format!("Invalid token! {}", e.to_string()).as_str()))?;

    serde_json::from_str::<UserClaims>(&decoded.claims.sub)
        .map_err(|e| ApiError::new(401, format!("Failed to decode claims! {}", e.to_string()).as_str()))
}