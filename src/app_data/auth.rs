use jsonwebtoken::{DecodingKey, EncodingKey};
use openidconnect::{core::CoreClient, EndpointNotSet, EndpointSet};
use std::sync::Arc;

use crate::{auth::discord::create_discord_client, get_secret};

pub type DiscordClient = CoreClient<
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
    EndpointNotSet,
>;

pub struct AuthAppState {
    pub discord_client: DiscordClient,
    pub jwt_encoding_key: EncodingKey,
    pub jwt_decoding_key: DecodingKey,
}

pub async fn init_app_state() -> Arc<AuthAppState> {
    let discord_client = create_discord_client()
        .await
        .expect("Failed to create discord client!");

    let jwt_secret = get_secret("JWT_SECRET");

    Arc::new(AuthAppState {
        discord_client,
        jwt_encoding_key: EncodingKey::from_base64_secret(jwt_secret.as_ref())
            .expect("Failed to create jwt encoding key"),
        jwt_decoding_key: DecodingKey::from_base64_secret(jwt_secret.as_ref())
            .expect("Failed to create jwt decoding key"),
    })
}
