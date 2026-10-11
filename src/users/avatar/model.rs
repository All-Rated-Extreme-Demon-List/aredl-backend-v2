use crate::app_data::db::DbAppState;
use crate::error_handler::{ApiError, ErrorResponse, StartupError};
use crate::providers::ProvidersAppState;
use crate::schema::users;
use crate::users::User;
use crate::{create_client, get_secret};
use actix_web::{http::StatusCode, web, HttpResponse, ResponseError};
use chrono::Utc;
use diesel::prelude::*;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use serde::Deserialize;
use std::{fmt, sync::Arc, time::Duration};
use tokio::{sync::Mutex, time::Instant};
use uuid::Uuid;

const DEFAULT_DELAY: Duration = Duration::from_millis(500);
const RATE_LIMIT_MARGIN: Duration = Duration::from_millis(50);

pub struct AvatarRefresher {
    client: reqwest::Client,
    discord_base: String,
    authorization: HeaderValue,
    next_request_at: Mutex<Instant>,
}

#[derive(Deserialize)]
struct DiscordAvatarDecorationData {
    asset: String,
}

#[derive(Deserialize)]
struct DiscordUser {
    avatar: Option<String>,
    avatar_decoration_data: Option<DiscordAvatarDecorationData>,
}

#[derive(Debug)]
pub enum RefreshError {
    Api(ApiError),
    RateLimited(Duration),
}

impl From<ApiError> for RefreshError {
    fn from(error: ApiError) -> Self {
        Self::Api(error)
    }
}

impl fmt::Display for RefreshError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Api(error) => error.fmt(f),
            Self::RateLimited(delay) => {
                write!(f, "Discord avatar refresh rate limited for {delay:?}")
            }
        }
    }
}

impl ResponseError for RefreshError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Api(error) => error.status_code(),
            Self::RateLimited(_) => StatusCode::TOO_MANY_REQUESTS,
        }
    }

    fn error_response(&self) -> HttpResponse {
        match self {
            Self::Api(error) => error.error_response(),
            Self::RateLimited(delay) => HttpResponse::TooManyRequests()
                .insert_header((
                    "Retry-After",
                    delay.as_secs_f64().ceil().max(1.0).to_string(),
                ))
                .json(ErrorResponse {
                    message: "Discord avatar refresh is rate limited, please try again later"
                        .into(),
                }),
        }
    }
}

impl AvatarRefresher {
    pub fn new(providers: &ProvidersAppState) -> Result<Self, StartupError> {
        let discord_base = providers.context.discord_auth.as_ref().map_or_else(
            || "https://discord.com".to_owned(),
            |auth| auth.api_base_uri.clone(),
        );
        let client = create_client().map_err(|error| {
            StartupError::Init(format!(
                "Failed to start Discord avatar refresh HTTP client: {error}"
            ))
        })?;
        let bot_token = get_secret("DISCORD_BOT_TOKEN")?;
        let mut authorization =
            HeaderValue::from_str(&format!("Bot {bot_token}")).map_err(|error| {
                StartupError::Init(format!("Invalid Discord bot authorization header: {error}"))
            })?;
        authorization.set_sensitive(true);
        Ok(Self {
            client,
            discord_base: discord_base.trim_end_matches('/').to_owned(),
            authorization,
            next_request_at: Mutex::new(Instant::now()),
        })
    }

    pub async fn wait_until_next(&self) {
        let next_request_at = *self.next_request_at.lock().await;
        tokio::time::sleep_until(next_request_at).await;
    }

    pub async fn refresh_user(
        &self,
        db: Arc<DbAppState>,
        user_id: Uuid,
    ) -> Result<User, RefreshError> {
        let db_user = db.clone();
        let discord_id = web::block(move || {
            User::from_uuid(&mut db_user.connection()?, user_id)?
                .discord_id
                .ok_or_else(|| ApiError::BadRequest("User has no linked Discord account"))
        })
        .await
        .map_err(ApiError::from)??;

        // only fetch if not rate limited
        let mut next_request_at = self.next_request_at.lock().await;
        let now = Instant::now();
        if *next_request_at > now {
            return Err(RefreshError::RateLimited(
                next_request_at.saturating_duration_since(now),
            ));
        }
        *next_request_at = now + DEFAULT_DELAY;

        // get discord user
        let response = self
            .client
            .get(format!("{}/api/v10/users/{discord_id}", self.discord_base))
            .header(AUTHORIZATION, self.authorization.clone())
            .timeout(Duration::from_secs(30))
            .send()
            .await
            .map_err(|error| {
                ApiError::BadGateway(format!("Discord avatar request failed: {error}"))
            })?;

        let response_status = response.status();
        let response_headers = response.headers().clone();
        let mut delay_until_next = Self::get_delay_to_await(&response_headers);

        // if rate limited
        if response_status == reqwest::StatusCode::TOO_MANY_REQUESTS {
            let body = response.json::<serde_json::Value>().await.ok();
            delay_until_next = body
                .as_ref()
                .and_then(|body| body.get("retry_after")?.as_f64())
                .and_then(|seconds| Duration::try_from_secs_f64(seconds).ok())
                .or_else(|| Self::parse_header_seconds(&response_headers, "retry-after"))
                .unwrap_or(DEFAULT_DELAY)
                .saturating_add(RATE_LIMIT_MARGIN);
            *next_request_at = Instant::now()
                .checked_add(delay_until_next)
                .unwrap_or_else(|| Instant::now() + DEFAULT_DELAY);
            return Err(RefreshError::RateLimited(delay_until_next));
        }
        *next_request_at = Instant::now()
            .checked_add(delay_until_next)
            .unwrap_or_else(|| Instant::now() + DEFAULT_DELAY);

        if !response_status.is_success() {
            return Err(ApiError::BadGateway(format!(
                "Discord avatar request returned {response_status}"
            ))
            .into());
        }

        let discord_user: DiscordUser = response.json().await.map_err(|error| {
            ApiError::BadGateway(format!("Failed to parse Discord user: {error}"))
        })?;

        drop(next_request_at);

        Ok(web::block(move || {
            diesel::update(
                users::table
                    .find(user_id)
                    .filter(users::discord_id.eq(discord_id)),
            )
            .set((
                users::discord_avatar.eq(discord_user.avatar),
                users::discord_avatar_decoration
                    .eq(discord_user.avatar_decoration_data.map(|data| data.asset)),
                users::last_discord_avatar_update.eq(Utc::now().naive_utc()),
            ))
            .returning(User::as_returning())
            .get_result(&mut db.connection()?)
            .map_err(ApiError::from)
        })
        .await
        .map_err(ApiError::from)??)
    }

    fn parse_header_seconds(headers: &HeaderMap, name: &str) -> Option<Duration> {
        let seconds = headers.get(name)?.to_str().ok()?.parse::<f64>().ok()?;
        Duration::try_from_secs_f64(seconds).ok()
    }

    fn get_delay_to_await(headers: &HeaderMap) -> Duration {
        let remaining = headers
            .get("x-ratelimit-remaining")
            .and_then(|value| value.to_str().ok()?.parse::<i64>().ok());
        if remaining.is_some_and(|remaining| remaining <= 1) {
            Self::parse_header_seconds(headers, "x-ratelimit-reset-after")
                .map_or(DEFAULT_DELAY, |delay| {
                    delay.saturating_add(RATE_LIMIT_MARGIN)
                })
        } else {
            DEFAULT_DELAY
        }
    }
}
