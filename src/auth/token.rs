use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::error_handler::ApiError;

#[derive(Debug, Serialize, Deserialize)]
pub struct UserClaims {
    pub user_id: Uuid,
    pub is_api_key: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TokenClaims {
    pub sub: UserClaims,
    pub iat: usize,
    pub exp: usize,
}

pub fn create_token(
    token_data: UserClaims,
    encoding_key: &EncodingKey,
    expires_in: Duration,
) -> Result<(String, DateTime<Utc>), ApiError> {

    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let expire_datetime = now + expires_in;
    let exp = expire_datetime.timestamp() as usize;
    let claims = TokenClaims {
        sub: token_data,
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