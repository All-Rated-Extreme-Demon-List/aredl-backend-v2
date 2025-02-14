use chrono::{DateTime, Duration, TimeZone, Utc};
use jsonwebtoken::{Algorithm, decode, DecodingKey, encode, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::{QueryDsl, ExpressionMethods, SelectableHelper, RunQueryDsl};
use crate::error_handler::ApiError;
use crate::schema::users;
use crate::db::DbConnection;
use crate::users::User;

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
    pub token_type: String,
}

pub fn create_token(
    user_claims: UserClaims,
    encoding_key: &EncodingKey,
    expires_in: Duration,
    token_type: &str,
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
        token_type: token_type.to_string(),
    };

    let token = encode(
        &Header::default(),
        &claims,
        &encoding_key,
    ).map_err(|_| ApiError::new(400, "Failed to create token!"))?;

    Ok((token, expire_datetime))
}

pub fn decode_token<T: Into<String>>(token: T, decoding_key: &DecodingKey, expected_types: &[&str]) -> Result<TokenClaims, ApiError> {
    let token_str = token.into();

    let decoded = decode::<TokenClaims>(
        &token_str,
        &decoding_key,
        &Validation::new(Algorithm::HS256),
    ).map_err(|e| ApiError::new(401, format!("Invalid token! {}", e.to_string()).as_str()))?;

    if !expected_types.is_empty() && !expected_types.contains(&decoded.claims.token_type.as_str()) {
        return Err(ApiError::new(401, "Invalid token type"));
    }

    Ok(decoded.claims)
}

pub fn decode_user_claims(token_claims: &TokenClaims) -> Result<UserClaims, ApiError> {
    serde_json::from_str(&token_claims.sub)
        .map_err(|e| ApiError::new(401, format!("Failed to decode user claims! {}", e.to_string()).as_str()))
}

pub fn check_token_valid(token_claims: &TokenClaims, user_claims: &UserClaims,conn: &mut DbConnection) -> Result<(), ApiError> {
    let user_record = users::table
        .filter(users::id.eq(user_claims.user_id))
        .select(User::as_select())
        .first::<User>(conn)
        .map_err(|_| ApiError::new(401, "User not found"))?;

    let token_issue_time = match Utc.timestamp_opt(token_claims.iat as i64, 0) {
        chrono::LocalResult::Single(dt) => dt.naive_utc(),
        _ => return Err(ApiError::new(401, "Invalid token issue time")),
    };

    if token_issue_time < user_record.access_valid_after {
        return Err(ApiError::new(401, "Token has been invalidated"));
    }

    Ok(())
}

pub fn create_test_token(user_id: Uuid, jwt_encoding_key: &EncodingKey) -> Result<String, ApiError> {
    let (token, _expires) = create_token(
        UserClaims {
            user_id,
            is_api_key: false,
        },
        jwt_encoding_key,
        Duration::minutes(30),
        "access",
    )?;

    Ok(token)
}