use crate::db::DbAppState;
use crate::get_secret;
use crate::schema::users;
use chrono::Utc;
use cron::Schedule;
use diesel::PgSortExpressionMethods;
use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, RunQueryDsl};
use reqwest::header::{HeaderMap, AUTHORIZATION};
use reqwest::StatusCode;
use serde::Deserialize;
use std::str::FromStr;
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
    if let Some(remaining) = header_i64(headers, "x-ratelimit-remaining") {
        if remaining <= 1 {
            if let Some(reset_after) = header_f64(headers, "x-ratelimit-reset-after") {
                let sleep_ms = (reset_after * 1000.0) as u64 + 50;
                tokio::time::sleep(Duration::from_millis(sleep_ms)).await;
                return;
            }
        }
    }
    tokio::time::sleep(Duration::from_millis(default_sleep_ms)).await;
}

pub async fn start_discord_avatars_refresher(db: Arc<DbAppState>) {
    let schedule = Schedule::from_str(&get_secret("DISCORD_AVATARS_REFRESH_SCHEDULE")).unwrap();
    let schedule = Arc::new(schedule);
    let db_clone = db.clone();

    let discord_base =
        std::env::var("DISCORD_BASE_URL").unwrap_or_else(|_| "https://discord.com".to_string());

    let client = reqwest::Client::builder()
        .user_agent("AredlBackend/2.0 (+https://api.aredl.net)")
        .build()
        .expect("Failed to build reqwest client");

    let discord_bot_token = get_secret("DISCORD_BOT_TOKEN");

    task::spawn(async move {
        loop {
            tracing::info!("Refreshing discord avatars");

            let mut conn = match db_clone.connection() {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("DB connection failed for avatar refresh: {e}");
                    continue;
                }
            };

            let users_to_refresh = users::table
                .filter(users::discord_id.is_not_null())
                .filter(
                    users::last_discord_avatar_update
                        .is_null()
                        .or(users::last_discord_avatar_update
                        .lt((Utc::now() - chrono::Duration::days(STALE_AFTER_DAYS)).naive_utc())),
                )
                .order(users::last_discord_avatar_update.asc().nulls_first())
                .limit(BATCH_LIMIT)
                .select((users::id, users::discord_id))
                .load::<(uuid::Uuid, Option<String>)>(&mut conn)
                .expect("Failed to load users for avatar refresh");

            if users_to_refresh.is_empty() {
                tracing::info!("No stale user avatars to refresh");
            } else {
                tracing::info!("Found {} users to refresh", users_to_refresh.len());

                for (user_id, discord_id) in users_to_refresh {
                    let Some(discord_id) = discord_id else {
                        continue;
                    };

                    let url = format!("{}/api/v10/users/{}", discord_base, discord_id);
                    let resp = match client
                        .get(&url)
                        .header(AUTHORIZATION, format!("Bot {}", discord_bot_token))
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
                        if let Ok(value) = resp.json::<serde_json::Value>().await {
                            if let Some(retry_after_s) =
                                value.get("retry_after").and_then(|v| v.as_f64())
                            {
                                let retry_after_ms = (retry_after_s * 1000.0) as u64;
                                tracing::warn!(
                                    "Rate limited by discord: waiting for {} ms",
                                    retry_after_ms
                                );
                                tokio::time::sleep(Duration::from_millis(retry_after_ms)).await;
                                continue;
                            }
                        } else {
                            tokio::time::sleep(Duration::from_millis(DEFAULT_DELAY_MS)).await;
                        }
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

                    match diesel::update(users::table.find(user_id))
                        .set((
                            users::discord_avatar.eq(updated_discord_user.avatar),
                            users::last_discord_avatar_update.eq(Utc::now().naive_utc()),
                        ))
                        .execute(&mut conn)
                    {
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

            let now = Utc::now();
            let next = schedule.upcoming(Utc).next().unwrap();
            let duration = next - now;

            tokio::time::sleep(Duration::from_secs(duration.num_seconds() as u64)).await;
        }
    });
}
