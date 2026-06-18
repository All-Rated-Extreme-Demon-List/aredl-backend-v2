use jsonwebtoken::{DecodingKey, EncodingKey};
use std::sync::Arc;

use crate::get_secret;

pub struct AuthAppState {
    pub jwt_encoding_key: EncodingKey,
    pub jwt_decoding_key: DecodingKey,
}

pub async fn init_app_state() -> Arc<AuthAppState> {
    let jwt_secret = get_secret("JWT_SECRET");

    Arc::new(AuthAppState {
        jwt_encoding_key: EncodingKey::from_base64_secret(jwt_secret.as_ref())
            .expect("Failed to create jwt encoding key"),
        jwt_decoding_key: DecodingKey::from_base64_secret(jwt_secret.as_ref())
            .expect("Failed to create jwt decoding key"),
    })
}
