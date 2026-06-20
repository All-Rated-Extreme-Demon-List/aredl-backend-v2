use jsonwebtoken::{DecodingKey, EncodingKey};
use std::sync::Arc;

use crate::error_handler::StartupError;
use crate::get_secret;

pub struct AuthAppState {
    pub jwt_encoding_key: EncodingKey,
    pub jwt_decoding_key: DecodingKey,
}

pub fn init_app_state() -> Result<Arc<AuthAppState>, StartupError> {
    let jwt_secret = get_secret("JWT_SECRET")?;

    Ok(Arc::new(AuthAppState {
        jwt_encoding_key: EncodingKey::from_base64_secret(jwt_secret.as_ref()).map_err(
            |error| StartupError::Init(format!("Failed to start JWT encoding key: {error}")),
        )?,
        jwt_decoding_key: DecodingKey::from_base64_secret(jwt_secret.as_ref()).map_err(
            |error| StartupError::Init(format!("Failed to start JWT decoding key: {error}")),
        )?,
    }))
}
