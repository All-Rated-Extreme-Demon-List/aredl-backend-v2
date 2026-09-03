use crate::app_data::db::DbAppState;
use crate::error_handler::{ApiError, StartupError};
use crate::get_secret;
use crate::providers::ProvidersAppState;
use crate::scheduled::{sleep_until_next, startup_schedule};
use crate::schema::aredl;
use crate::schema::arepl;
use chrono::Utc;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;
use uuid::Uuid;

use diesel::prelude::*;
pub async fn start_level_data_refresher(
    db: Arc<DbAppState>,
    providers: Arc<ProvidersAppState>,
) -> Result<(), StartupError> {
    let schedule = startup_schedule("LEVEL_DATA_REFRESH_SCHEDULE")?;

    let edel_sheet_id = get_secret("EDEL_SHEET_ID")?;
    let nlw_sheet_id = get_secret("NLW_SHEET_ID")?;

    let Some(google_auth) = providers.context.google_auth.clone() else {
        tracing::warn!("Failed to refresh level data: Google OAuth is not configured");
        return Ok(());
    };

    let db_clone = db.clone();
    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            tracing::info!("Refreshing level data");

            let google_access_token = match google_auth.get_access_token(&db_clone).await {
                Ok(token) => token,
                Err(e) => {
                    tracing::error!("Failed to get Google access token: {e}");
                    continue;
                }
            };

            if let Err(error) =
                update_edel_data(&db_clone, &google_access_token, &edel_sheet_id).await
            {
                tracing::error!("Failed to refresh edel {error}");
            }

            if let Err(error) =
                update_nlw_data(&db_clone, &google_access_token, &nlw_sheet_id).await
            {
                tracing::error!("Failed to refresh nlw {error}");
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

            if let Ok(list) = db.connection().and_then(|mut conn| {
                aredl::levels::table
                    .left_join(
                        aredl::last_gddl_update::table
                            .on(aredl::last_gddl_update::id.eq(aredl::levels::id)),
                    )
                    .filter(
                        aredl::last_gddl_update::updated_at
                            .is_null()
                            .or(aredl::last_gddl_update::updated_at.lt(one_day_ago)),
                    )
                    .select((
                        aredl::levels::id,
                        aredl::levels::level_id,
                        aredl::levels::two_player,
                    ))
                    .load::<(Uuid, i32, bool)>(&mut conn)
                    .map_err(ApiError::from)
            }) {
                for (id, level_id, two_p) in list {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    if let Err(e) = aredl_update_gddl_data(&db, id, level_id, two_p).await {
                        tracing::error!("AREDL GDDL {} failed: {}", level_id, e);
                    }
                }
            }

            sleep_until_next(&schedule).await;
        }
    });

    Ok(())
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
#[diesel(table_name = aredl::levels)]
struct AredlEdelUpdate {
    id: Uuid,
    edel_enjoyment: Option<f64>,
    is_edel_pending: bool,
}

#[derive(AsChangeset, Identifiable)]
#[diesel(treat_none_as_null = true)]
#[diesel(table_name = arepl::levels)]
struct AreplEdelUpdate {
    id: Uuid,
    edel_enjoyment: Option<f64>,
    is_edel_pending: bool,
}

#[derive(AsChangeset, Identifiable)]
#[diesel(treat_none_as_null = true)]
#[diesel(table_name = aredl::levels)]
struct AredlNlwTierUpdate {
    id: Uuid,
    nlw_tier: Option<String>,
}

#[derive(AsChangeset, Identifiable)]
#[diesel(treat_none_as_null = true)]
#[diesel(table_name = arepl::levels)]
struct AreplNlwTierUpdate {
    id: Uuid,
    nlw_tier: Option<String>,
}

async fn aredl_update_gddl_data(
    db: &DbAppState,
    id: Uuid,
    level_id: i32,
    two_player: bool,
) -> Result<(), ApiError> {
    let url = format!("https://gdladder.com/api/level/{level_id}");

    let client = reqwest::Client::builder()
        .user_agent("AredlBackend/2.0 (+https://api.aredl.net)")
        .build()
        .map_err(|e| {
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

    let conn = &mut db.connection()?;

    diesel::update(aredl::levels::table)
        .filter(aredl::levels::id.eq(id))
        .set(aredl::levels::gddl_tier.eq(rating))
        .execute(conn)?;

    diesel::insert_into(aredl::last_gddl_update::table)
        .values((
            aredl::last_gddl_update::id.eq(id),
            aredl::last_gddl_update::updated_at.eq(Utc::now()),
        ))
        .on_conflict(aredl::last_gddl_update::id)
        .do_update()
        .set(aredl::last_gddl_update::updated_at.eq(Utc::now()))
        .execute(conn)?;

    Ok(())
}

async fn update_edel_data(
    db: &DbAppState,
    access_token: &str,
    spreadsheet_id: &str,
) -> Result<(), ApiError> {
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

    let conn = &mut db.connection()?;

    conn.transaction(|conn| {
        let aredl_levels = aredl::levels::table
            .filter(
                aredl::levels::level_id
                    .eq_any(&level_ids)
                    .or(aredl::levels::edel_enjoyment
                        .is_not_null()
                        .or(aredl::levels::is_edel_pending.eq(true))),
            )
            .select((aredl::levels::id, aredl::levels::level_id))
            .load::<(Uuid, i32)>(conn)?;
        let aredl_updates = aredl_levels
            .into_iter()
            .map(|(id, level_id)| {
                let (edel_enjoyment, is_edel_pending) = data
                    .get(&level_id)
                    .map_or((None, false), |(enjoyment, pending)| {
                        (Some(*enjoyment), *pending)
                    });

                AredlEdelUpdate {
                    id,
                    edel_enjoyment,
                    is_edel_pending,
                }
            })
            .collect::<Vec<_>>();

        if !aredl_updates.is_empty() {
            diesel::update(aredl::levels::table)
                .set(&aredl_updates)
                .execute(conn)?;
        }

        let arepl_levels = arepl::levels::table
            .filter(
                arepl::levels::level_id
                    .eq_any(&level_ids)
                    .or(arepl::levels::edel_enjoyment
                        .is_not_null()
                        .or(arepl::levels::is_edel_pending.eq(true))),
            )
            .select((arepl::levels::id, arepl::levels::level_id))
            .load::<(Uuid, i32)>(conn)?;
        let arepl_updates = arepl_levels
            .into_iter()
            .map(|(id, level_id)| {
                let (edel_enjoyment, is_edel_pending) = data
                    .get(&level_id)
                    .map_or((None, false), |(enjoyment, pending)| {
                        (Some(*enjoyment), *pending)
                    });

                AreplEdelUpdate {
                    id,
                    edel_enjoyment,
                    is_edel_pending,
                }
            })
            .collect::<Vec<_>>();

        if !arepl_updates.is_empty() {
            diesel::update(arepl::levels::table)
                .set(&arepl_updates)
                .execute(conn)?;
        }

        Ok(())
    })
}

async fn update_nlw_data(
    db: &DbAppState,
    access_token: &str,
    spreadsheet_id: &str,
) -> Result<(), ApiError> {
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

    let conn = &mut db.connection()?;

    conn.transaction(|conn| {
        let aredl_levels = aredl::levels::table
            .filter(
                aredl::levels::level_id
                    .eq_any(&level_ids)
                    .or(aredl::levels::nlw_tier.is_not_null()),
            )
            .select((aredl::levels::id, aredl::levels::level_id))
            .load::<(Uuid, i32)>(conn)?;
        let aredl_updates = aredl_levels
            .into_iter()
            .map(|(id, level_id)| AredlNlwTierUpdate {
                id,
                nlw_tier: data.get(&level_id).cloned(),
            })
            .collect::<Vec<_>>();

        if !aredl_updates.is_empty() {
            diesel::update(aredl::levels::table)
                .set(&aredl_updates)
                .execute(conn)?;
        }

        let arepl_levels = arepl::levels::table
            .filter(
                arepl::levels::level_id
                    .eq_any(&level_ids)
                    .or(arepl::levels::nlw_tier.is_not_null()),
            )
            .select((arepl::levels::id, arepl::levels::level_id))
            .load::<(Uuid, i32)>(conn)?;
        let arepl_updates = arepl_levels
            .into_iter()
            .map(|(id, level_id)| AreplNlwTierUpdate {
                id,
                nlw_tier: data.get(&level_id).cloned(),
            })
            .collect::<Vec<_>>();

        if !arepl_updates.is_empty() {
            diesel::update(arepl::levels::table)
                .set(&arepl_updates)
                .execute(conn)?;
        }

        Ok(())
    })
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
