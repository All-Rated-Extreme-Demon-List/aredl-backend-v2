use crate::app_data::db::DbAppState;
use crate::error_handler::{ApiError, StartupError};
use crate::get_secret;
use crate::providers::ProvidersAppState;
use crate::scheduled::{sleep_until_next, startup_schedule};
use crate::schema::users;
use chrono::Utc;
use diesel::PgSortExpressionMethods as _;
use diesel::{BoolExpressionMethods as _, ExpressionMethods as _, QueryDsl as _, RunQueryDsl as _};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use reqwest::StatusCode;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;

const BATCH_LIMIT: i64 = 200;
const STALE_AFTER_DAYS: i64 = 14;
const DEFAULT_DELAY_MS: u64 = 500;

#[derive(Deserialize)]
struct DiscordUser {
    avatar: Option<String>,
}

fn header_i64(headers: &HeaderMap, name: &str) -> Option<i64> {
    headers.get(name)?.to_str().ok()?.parse::<i64>().ok()
}
fn header_f64(headers: &HeaderMap, name: &str) -> Option<f64> {
    headers.get(name)?.to_str().ok()?.parse::<f64>().ok()
}

async fn sleep_for_ratelimit(headers: &HeaderMap, default_sleep_ms: u64) {
    let default_sleep = Duration::from_millis(default_sleep_ms);
    let sleep_duration = match (
        header_i64(headers, "x-ratelimit-remaining"),
        header_f64(headers, "x-ratelimit-reset-after"),
    ) {
        (Some(remaining), Some(reset_after)) if remaining <= 1 => {
            Duration::try_from_secs_f64(reset_after).map_or(default_sleep, |duration| {
                duration.saturating_add(Duration::from_millis(50))
            })
        }
        _ => default_sleep,
    };

    tokio::time::sleep(sleep_duration).await;
}

pub async fn start_discord_avatars_refresher(
    db: Arc<DbAppState>,
    providers: Arc<ProvidersAppState>,
) -> Result<(), StartupError> {
    let schedule = startup_schedule("DISCORD_AVATARS_REFRESH_SCHEDULE")?;

    let discord_base = providers.context.discord_auth.as_ref().map_or_else(
        || "https://discord.com".to_owned(),
        |discord_auth| discord_auth.api_base_uri.clone(),
    );

    let client = reqwest::Client::builder()
        .user_agent("AredlBackend/2.0 (+https://api.aredl.net)")
        .build()
        .map_err(|error| {
            StartupError::Init(format!(
                "Failed to start Discord avatar refresh HTTP client: {error}"
            ))
        })?;

    let discord_bot_token = get_secret("DISCORD_BOT_TOKEN")?;

    task::spawn(async move {
        loop {
            tracing::info!("Refreshing discord avatars");

            let users_to_refresh = match db.connection().and_then(|mut conn| {
                users::table
                    .filter(users::discord_id.is_not_null())
                    .filter(users::last_discord_avatar_update.is_null().or(
                        users::last_discord_avatar_update.lt(
                            (Utc::now() - chrono::Duration::days(STALE_AFTER_DAYS)).naive_utc(),
                        ),
                    ))
                    .order(users::last_discord_avatar_update.asc().nulls_first())
                    .limit(BATCH_LIMIT)
                    .select((users::id, users::discord_id))
                    .load::<(uuid::Uuid, Option<String>)>(&mut conn)
                    .map_err(ApiError::from)
            }) {
                Ok(users) => users,
                Err(error) => {
                    tracing::error!("Failed to load users for avatar refresh: {error}");
                    sleep_until_next(&schedule).await;
                    continue;
                }
            };

            if users_to_refresh.is_empty() {
                tracing::info!("No stale user avatars to refresh");
            } else {
                tracing::info!("Found {} users to refresh", users_to_refresh.len());

                for (user_id, discord_id) in users_to_refresh {
                    let Some(discord_id) = discord_id else {
                        continue;
                    };

                    let url = format!("{discord_base}/api/v10/users/{discord_id}");
                    let resp = match client
                        .get(&url)
                        .header(AUTHORIZATION, format!("Bot {discord_bot_token}"))
                        .send()
                        .await
                    {
                        Ok(r) => r,
                        Err(e) => {
                            tracing::warn!(
                                "Failed to request discord avatar for {}: {}",
                                discord_id,
                                e
                            );
                            tokio::time::sleep(Duration::from_millis(DEFAULT_DELAY_MS)).await;
                            continue;
                        }
                    };

                    if resp.status() == StatusCode::TOO_MANY_REQUESTS {
                        let retry_after = match resp.json::<serde_json::Value>().await {
                            Ok(value) => value
                                .get("retry_after")
                                .and_then(serde_json::Value::as_f64)
                                .and_then(|seconds| Duration::try_from_secs_f64(seconds).ok())
                                .map(|duration| duration.saturating_add(Duration::from_millis(50))),
                            Err(error) => {
                                tracing::warn!(
                                    "Failed to parse Discord rate-limit response: {error}"
                                );
                                None
                            }
                        }
                        .unwrap_or(Duration::from_millis(DEFAULT_DELAY_MS));

                        tracing::warn!(
                            "Rate limited by Discord: waiting for {} ms",
                            retry_after.as_millis()
                        );

                        tokio::time::sleep(retry_after).await;
                        continue;
                    }

                    if !resp.status().is_success() {
                        tracing::warn!(
                            "Failed to refresh avatar for {}: {}",
                            discord_id,
                            resp.status()
                        );
                        tokio::time::sleep(Duration::from_millis(DEFAULT_DELAY_MS)).await;
                        continue;
                    }

                    let headers = resp.headers().clone();

                    let updated_discord_user: DiscordUser = match resp.json().await {
                        Ok(j) => j,
                        Err(e) => {
                            tracing::warn!(
                                "Failed to parse discord user for {}: {}",
                                discord_id,
                                e
                            );
                            tokio::time::sleep(Duration::from_millis(DEFAULT_DELAY_MS)).await;
                            continue;
                        }
                    };

                    match db.connection().and_then(|mut conn| {
                        diesel::update(users::table.find(user_id))
                            .set((
                                users::discord_avatar.eq(updated_discord_user.avatar),
                                users::last_discord_avatar_update.eq(Utc::now().naive_utc()),
                            ))
                            .execute(&mut conn)
                            .map_err(ApiError::from)
                    }) {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::warn!(
                                "Failed to update user avatar for {}: {}",
                                discord_id,
                                e
                            );
                        }
                    }

                    sleep_for_ratelimit(&headers, DEFAULT_DELAY_MS).await;
                }
            }

            sleep_until_next(&schedule).await;
        }
    });

    Ok(())
}
