use crate::app_data::db::DbAppState;
use crate::error_handler::{ApiError, StartupError};
use crate::get_secret;
use crate::providers::ProvidersAppState;
use crate::scheduled::{parse_startup_schedule, sleep_until_next, startup_schedule};
use crate::schema::aredl;
use crate::schema::arepl;
use chrono::Utc;
use diesel::dsl::exists;
use diesel::{
    select, BoolExpressionMethods, Connection, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl,
};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;
use uuid::Uuid;

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

    let schedule = parse_startup_schedule("GDDL updater schedule", "@hourly")?;

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

async fn aredl_update_gddl_data(
    db: &DbAppState,
    id: Uuid,
    level_id: i32,
    two_player: bool,
) -> Result<(), ApiError> {
    let url = format!("https://gdladder.com/api/level/{}", level_id);

    let client = reqwest::Client::builder()
        .user_agent("AredlBackend/2.0 (+https://api.aredl.net)")
        .build()
        .map_err(|e| {
            ApiError::InternalServerError(format!("Failed to build HTTP client: {:?}", e).as_str())
        })?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError::BadGateway(format!("Request failed: {:?}", e)))?
        .error_for_status()
        .map_err(|e| ApiError::BadGateway(format!("HTTP error: {:?}", e)))?;

    let data: GDDLResponse = response
        .json()
        .await
        .map_err(|e| ApiError::BadGateway(format!("Failed to request gddl: {:?}", e)))?;

    let rating = match (two_player, data.two_player_rating, data.rating) {
        (true, Some(two_player_rating), _) => Some(two_player_rating),
        (true, None, _) => data.default_rating,
        (false, _, Some(rating)) => Some(rating),
        (false, _, None) => data.default_rating,
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

    let data: Vec<(i32, f64, bool)> = ids_result
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
        .collect();

    let conn = &mut db.connection()?;

    conn.transaction(|conn| {
        for (level_id, enjoyment, pending) in &data {
            let aredl_2p: bool = select(exists(
                aredl::levels::table
                    .filter(aredl::levels::level_id.eq(*level_id))
                    .filter(aredl::levels::two_player.eq(true)),
            ))
            .get_result(conn)?;
            diesel::update(aredl::levels::table)
                .set((
                    aredl::levels::edel_enjoyment.eq(*enjoyment),
                    aredl::levels::is_edel_pending.eq(*pending),
                ))
                .filter(aredl::levels::level_id.eq(*level_id))
                .filter(aredl::levels::two_player.eq(aredl_2p))
                .execute(conn)?;

            let arepl_2p: bool = select(exists(
                arepl::levels::table
                    .filter(arepl::levels::level_id.eq(*level_id))
                    .filter(arepl::levels::two_player.eq(true)),
            ))
            .get_result(conn)?;
            diesel::update(arepl::levels::table)
                .set((
                    arepl::levels::edel_enjoyment.eq(*enjoyment),
                    arepl::levels::is_edel_pending.eq(*pending),
                ))
                .filter(arepl::levels::level_id.eq(*level_id))
                .filter(arepl::levels::two_player.eq(arepl_2p))
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

    let data: Vec<(i32, String)> = ids_result
        .values
        .into_iter()
        .filter_map(|values| -> Option<(i32, String)> {
            let [id, tier, ..] = values.as_slice() else {
                return None;
            };

            Some((id.parse().ok()?, tier.clone()))
        })
        .collect();

    let conn = &mut db.connection()?;

    conn.transaction(|conn| {
        for (level_id, tier) in &data {
            let aredl_2p: bool = select(exists(
                aredl::levels::table
                    .filter(aredl::levels::level_id.eq(*level_id))
                    .filter(aredl::levels::two_player.eq(true)),
            ))
            .get_result(conn)?;
            diesel::update(aredl::levels::table)
                .set(aredl::levels::nlw_tier.eq(tier))
                .filter(aredl::levels::level_id.eq(*level_id))
                .filter(aredl::levels::two_player.eq(aredl_2p))
                .execute(conn)?;

            let arepl_2p: bool = select(exists(
                arepl::levels::table
                    .filter(arepl::levels::level_id.eq(*level_id))
                    .filter(arepl::levels::two_player.eq(true)),
            ))
            .get_result(conn)?;
            diesel::update(arepl::levels::table)
                .set(arepl::levels::nlw_tier.eq(tier))
                .filter(arepl::levels::level_id.eq(*level_id))
                .filter(arepl::levels::two_player.eq(arepl_2p))
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
    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}",
        spreadsheet_id, range
    );
    let response = reqwest::Client::new()
        .get(&url)
        .bearer_auth(access_token)
        .send()
        .await
        .map_err(|e| {
            ApiError::BadGateway(format!("Failed to request spreadsheet: {}", e).as_str())
        })?;
    if !response.status().is_success() {
        return Err(ApiError::BadGateway("Failed to request spreadsheet"));
    }

    let sheet_values: SheetValues = response.json().await.map_err(|e| {
        ApiError::BadGateway(format!("Failed to request spreadsheet: {}", e).as_str())
    })?;

    Ok(sheet_values)
}
