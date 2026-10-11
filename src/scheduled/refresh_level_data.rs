use crate::app_data::db::DbAppState;
use crate::error_handler::{ApiError, StartupError};
use crate::list::List;
use crate::providers::ProvidersAppState;
use crate::scheduled::{sleep_until_next, startup_schedule};
use crate::schema::{last_gddl_update, levels};
use crate::{create_client, get_secret};
use actix_web::web;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;

#[derive(Serialize, ToSchema)]
pub struct LevelDataRefreshResult {
    pub updated_levels: usize,
}

pub async fn start_level_data_refresher(
    db: Arc<DbAppState>,
    providers: Arc<ProvidersAppState>,
) -> Result<(), StartupError> {
    let schedule = startup_schedule("LEVEL_DATA_REFRESH_SCHEDULE")?;
    if providers.context.google_auth.is_none() {
        tracing::warn!("Failed to refresh level data: Google OAuth is not configured");
        return Ok(());
    }
    let db_clone = db.clone();
    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;
            if let Err(error) = refresh_edel(db_clone.clone(), providers.clone()).await {
                tracing::error!("Failed to refresh EDEL: {error}");
            }
            if let Err(error) = refresh_nlw(db_clone.clone(), providers.clone()).await {
                tracing::error!("Failed to refresh NLW: {error}");
            }
            sleep_until_next(&schedule).await;
        }
    });

    let schedule = startup_schedule("LEVEL_DATA_REFRESH_SCHEDULE")?;

    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            tracing::info!("Running gddl updater");

            let one_day_ago = Utc::now() - chrono::Duration::days(1);

            if let Ok(levels_to_update) = db.connection().and_then(|mut conn| {
                levels::table
                    .left_join(last_gddl_update::table)
                    .filter(
                        last_gddl_update::updated_at
                            .is_null()
                            .or(last_gddl_update::updated_at.lt(one_day_ago)),
                    )
                    .select((
                        levels::list_id,
                        levels::id,
                        levels::level_id,
                        levels::two_player,
                    ))
                    .load::<(List, Uuid, i32, bool)>(&mut conn)
                    .map_err(ApiError::from)
            }) {
                for (list, id, level_id, two_p) in levels_to_update {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    if let Err(e) = update_gddl_data(db.clone(), id, list, level_id, two_p).await {
                        tracing::error!("GDDL {} failed: {}", level_id, e);
                    }
                }
            }

            sleep_until_next(&schedule).await;
        }
    });

    Ok(())
}

pub async fn refresh_edel(
    db: Arc<DbAppState>,
    providers: Arc<ProvidersAppState>,
) -> Result<LevelDataRefreshResult, ApiError> {
    let token = get_google_token(&db, &providers).await?;
    let sheet_id = get_secret("EDEL_SHEET_ID").map_err(ApiError::InternalServerError)?;
    let updated_levels = update_edel_data(db, &token, &sheet_id).await?;
    Ok(LevelDataRefreshResult { updated_levels })
}

pub async fn refresh_nlw(
    db: Arc<DbAppState>,
    providers: Arc<ProvidersAppState>,
) -> Result<LevelDataRefreshResult, ApiError> {
    let token = get_google_token(&db, &providers).await?;
    let sheet_id = get_secret("NLW_SHEET_ID").map_err(ApiError::InternalServerError)?;
    let updated_levels = update_nlw_data(db, &token, &sheet_id).await?;
    Ok(LevelDataRefreshResult { updated_levels })
}

pub async fn refresh_gddl(db: Arc<DbAppState>, list: List, id: Uuid) -> Result<(), ApiError> {
    let db_level = db.clone();
    let (level_id, two_player) = web::block(move || {
        levels::table
            .filter(levels::id.eq(id))
            .filter(levels::list_id.eq(list))
            .select((levels::level_id, levels::two_player))
            .first::<(i32, bool)>(&mut db_level.connection()?)
            .map_err(ApiError::from)
    })
    .await??;
    update_gddl_data(db, id, list, level_id, two_player).await
}

#[derive(Deserialize)]
struct GDDLResponse {
    #[serde(rename = "Rating")]
    rating: Option<f64>,
    #[serde(rename = "DefaultRating")]
    default_rating: Option<f64>,
    #[serde(rename = "TwoPlayerRating")]
    two_player_rating: Option<f64>,
}

#[derive(AsChangeset, Identifiable)]
#[diesel(treat_none_as_null = true)]
#[diesel(table_name = levels)]
struct EdelUpdate {
    id: Uuid,
    edel_enjoyment: Option<f64>,
    is_edel_pending: bool,
}

#[derive(AsChangeset, Identifiable)]
#[diesel(treat_none_as_null = true)]
#[diesel(table_name = levels)]
struct NlwTierUpdate {
    id: Uuid,
    nlw_tier: Option<String>,
}

async fn update_gddl_data(
    db: Arc<DbAppState>,
    id: Uuid,
    list: List,
    level_id: i32,
    two_player: bool,
) -> Result<(), ApiError> {
    let url = format!("https://gdladder.com/api/levels/{level_id}");

    let client = create_client().map_err(|e| {
        ApiError::InternalServerError(format!("Failed to build HTTP client: {e:?}").as_str())
    })?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError::BadGateway(format!("Request failed: {e:?}")))?
        .error_for_status()
        .map_err(|e| ApiError::BadGateway(format!("HTTP error: {e:?}")))?;

    let data: GDDLResponse = response
        .json()
        .await
        .map_err(|e| ApiError::BadGateway(format!("Failed to request gddl: {e:?}")))?;

    let rating = match (two_player, data.two_player_rating, data.rating) {
        (true, Some(two_player_rating), _) => Some(two_player_rating),
        (false, _, Some(rating)) => Some(rating),
        (_, _, _) => data.default_rating,
    };

    web::block(move || {
        let conn = &mut db.connection()?;

        diesel::update(levels::table)
            .filter(levels::id.eq(id))
            .set(levels::gddl_tier.eq(rating))
            .execute(conn)?;

        diesel::insert_into(last_gddl_update::table)
            .values((
                last_gddl_update::list_id.eq(list),
                last_gddl_update::id.eq(id),
                last_gddl_update::updated_at.eq(Utc::now()),
            ))
            .on_conflict(last_gddl_update::id)
            .do_update()
            .set(last_gddl_update::updated_at.eq(Utc::now()))
            .execute(conn)?;

        Ok(())
    })
    .await?
}

async fn update_edel_data(
    db: Arc<DbAppState>,
    access_token: &str,
    spreadsheet_id: &str,
) -> Result<usize, ApiError> {
    let ids_result = read_spreadsheet(access_token, spreadsheet_id, "'IDS'!B:D").await?;

    let data = ids_result
        .values
        .into_iter()
        .filter_map(|values| -> Option<(i32, f64, bool)> {
            let [id, enjoyment, pending, ..] = values.as_slice() else {
                return None;
            };
            Some((
                id.parse().ok()?,
                enjoyment.parse().ok()?,
                pending.parse().unwrap_or(false),
            ))
        })
        .map(|(level_id, enjoyment, pending)| (level_id, (enjoyment, pending)))
        .collect::<HashMap<_, _>>();
    let level_ids = data.keys().copied().collect::<Vec<_>>();

    web::block(move || {
        let conn = &mut db.connection()?;

        conn.transaction(|conn| {
            let levels_to_update = levels::table
                .filter(
                    levels::level_id
                        .eq_any(&level_ids)
                        .or(levels::edel_enjoyment
                            .is_not_null()
                            .or(levels::is_edel_pending.eq(true))),
                )
                .select((levels::id, levels::level_id))
                .load::<(Uuid, i32)>(conn)?;

            let levels_to_update_count = levels_to_update.len();

            let edel_updates = levels_to_update
                .into_iter()
                .map(|(id, level_id)| {
                    let (edel_enjoyment, is_edel_pending) = data
                        .get(&level_id)
                        .map_or((None, false), |(enjoyment, pending)| {
                            (Some(*enjoyment), *pending)
                        });

                    EdelUpdate {
                        id,
                        edel_enjoyment,
                        is_edel_pending,
                    }
                })
                .collect::<Vec<_>>();

            if !edel_updates.is_empty() {
                diesel::update(levels::table)
                    .set(&edel_updates)
                    .execute(conn)?;
            }

            Ok(levels_to_update_count)
        })
    })
    .await?
}

async fn update_nlw_data(
    db: Arc<DbAppState>,
    access_token: &str,
    spreadsheet_id: &str,
) -> Result<usize, ApiError> {
    let ids_result = read_spreadsheet(access_token, spreadsheet_id, "'IDS'!C:D").await?;

    let data = ids_result
        .values
        .into_iter()
        .filter_map(|values| -> Option<(i32, String)> {
            let [id, tier, ..] = values.as_slice() else {
                return None;
            };

            Some((id.parse().ok()?, tier.clone()))
        })
        .collect::<HashMap<_, _>>();
    let level_ids = data.keys().copied().collect::<Vec<_>>();

    web::block(move || {
        let conn = &mut db.connection()?;

        conn.transaction(|conn| {
            let levels_to_update = levels::table
                .filter(
                    levels::level_id
                        .eq_any(&level_ids)
                        .or(levels::nlw_tier.is_not_null()),
                )
                .select((levels::id, levels::level_id))
                .load::<(Uuid, i32)>(conn)?;
            let levels_to_update_count = levels_to_update.len();

            let nlw_updates = levels_to_update
                .into_iter()
                .map(|(id, level_id)| NlwTierUpdate {
                    id,
                    nlw_tier: data.get(&level_id).cloned(),
                })
                .collect::<Vec<_>>();

            if !nlw_updates.is_empty() {
                diesel::update(levels::table)
                    .set(&nlw_updates)
                    .execute(conn)?;
            }

            Ok(levels_to_update_count)
        })
    })
    .await?
}

#[derive(Deserialize)]
struct SheetValues {
    values: Vec<Vec<String>>,
}

async fn read_spreadsheet(
    access_token: &str,
    spreadsheet_id: &str,
    range: &str,
) -> Result<SheetValues, ApiError> {
    let url =
        format!("https://sheets.googleapis.com/v4/spreadsheets/{spreadsheet_id}/values/{range}");
    let response = reqwest::Client::new()
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            ApiError::BadGateway(format!("Failed to request spreadsheet: {e}").as_str())
        })?;
    if !response.status().is_success() {
        return Err(ApiError::BadGateway("Failed to request spreadsheet"));
    }

    let sheet_values: SheetValues = response.json().await.map_err(|e| {
        ApiError::BadGateway(format!("Failed to request spreadsheet: {e}").as_str())
    })?;

    Ok(sheet_values)
}

async fn get_google_token(
    db: &DbAppState,
    providers: &ProvidersAppState,
) -> Result<String, ApiError> {
    let auth = providers
        .context
        .google_auth
        .as_ref()
        .ok_or_else(|| ApiError::BadRequest("Google auth is not configured"))?;
    auth.get_access_token(db).await
}
