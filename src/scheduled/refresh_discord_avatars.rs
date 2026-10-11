use crate::app_data::db::DbAppState;
use crate::error_handler::{ApiError, StartupError};
use crate::scheduled::{sleep_until_next, startup_schedule};
use crate::schema::users;
use crate::users::avatar::{AvatarRefresher, RefreshError};
use actix_web::web;
use chrono::Utc;
use diesel::prelude::*;
use std::sync::Arc;
use tokio::task;
use uuid::Uuid;

const BATCH_LIMIT: i64 = 200;
const STALE_AFTER_DAYS: i64 = 14;

pub async fn start_discord_avatars_refresher(
    db: Arc<DbAppState>,
    refresher: Arc<AvatarRefresher>,
) -> Result<(), StartupError> {
    let schedule = startup_schedule("DISCORD_AVATARS_REFRESH_SCHEDULE")?;
    task::spawn(async move {
        loop {
            tracing::info!("Refreshing discord avatars");
            let db_users = db.clone();
            let users_to_refresh = web::block(move || {
                users::table
                    .filter(users::discord_id.is_not_null())
                    .filter(users::last_discord_avatar_update.is_null().or(
                        users::last_discord_avatar_update.lt(
                            (Utc::now() - chrono::Duration::days(STALE_AFTER_DAYS)).naive_utc(),
                        ),
                    ))
                    .order(users::last_discord_avatar_update.asc().nulls_first())
                    .limit(BATCH_LIMIT)
                    .select(users::id)
                    .load::<Uuid>(&mut db_users.connection()?)
                    .map_err(ApiError::from)
            })
            .await
            .map_err(ApiError::from)
            .and_then(|result| result);

            match users_to_refresh {
                Ok(user_ids) => {
                    tracing::info!("Found {} users to refresh", user_ids.len());
                    for user_id in user_ids {
                        loop {
                            refresher.wait_until_next().await;
                            match refresher.refresh_user(db.clone(), user_id).await {
                                Ok(_) => break,
                                Err(RefreshError::RateLimited(delay)) => {
                                    tracing::warn!(
                                        "Discord avatar refresh rate limited for {delay:?}"
                                    );
                                    tokio::time::sleep(delay).await;
                                }
                                Err(error) => {
                                    tracing::warn!(
                                        "Failed to refresh Discord avatar for {user_id}: {error}"
                                    );
                                    break;
                                }
                            }
                        }
                    }
                }
                Err(error) => {
                    tracing::error!("Failed to load users to refresh discord avatars: {error}");
                }
            }
            sleep_until_next(&schedule).await;
        }
    });
    Ok(())
}
