use crate::db::{DbAppState, DbConnection};
use crate::error_handler::ApiError;
use crate::get_secret;
use crate::schema::aredl;
use crate::schema::arepl;
use chrono::Utc;
use cron::Schedule;
use diesel::dsl::exists;
use diesel::{
    select, BoolExpressionMethods, Connection, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl,
};
use serde::Deserialize;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use tokio::task;
use uuid::Uuid;

pub async fn start_level_data_refresher(db: Arc<DbAppState>) {
    let schedule = Schedule::from_str(&get_secret("LEVEL_DATA_REFRESH_SCHEDULE")).unwrap();
    let schedule = Arc::new(schedule);

    let google_api_key = get_secret("GOOGLE_API_KEY");
    let edel_sheet_id = get_secret("EDEL_SHEET_ID");
    let nlw_sheet_id = get_secret("NLW_SHEET_ID");

    let db_clone = db.clone();
    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            tracing::info!("Refreshing level data");

            let conn = &mut match db.connection() {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("DB connection failed: {e}");
                    continue;
                }
            };

            let edel_result = update_edel_data(conn, &google_api_key, &edel_sheet_id).await;

            if edel_result.is_err() {
                tracing::error!("Failed to refresh edel {}", edel_result.err().unwrap());
            }

            let nlw_result = update_nlw_data(conn, &google_api_key, &nlw_sheet_id).await;

            if nlw_result.is_err() {
                tracing::error!("Failed to refresh nlw {}", nlw_result.err().unwrap());
            }

            let now = Utc::now();
            let next = schedule.upcoming(Utc).next().unwrap();
            let duration = next - now;

            tokio::time::sleep(Duration::from_secs(duration.num_seconds() as u64)).await;
        }
    });

    let schedule = Schedule::from_str("@hourly").expect("Failed to parse schedule");
    let schedule = Arc::new(schedule);

    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            tracing::info!("Running gddl updater");

            let conn = &mut match db_clone.connection() {
                Ok(c) => c,
                Err(e) => {
                    tracing::error!("DB connection failed: {e}");
                    continue;
                }
            };

            let one_day_ago = Utc::now() - chrono::Duration::days(1);

            if let Ok(list) = aredl::levels::table
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
                .load::<(Uuid, i32, bool)>(conn)
            {
                for (id, level_id, two_p) in list {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    if let Err(e) = aredl_update_gddl_data(conn, id, level_id, two_p).await {
                        tracing::error!("AREDL GDDL {} failed: {}", level_id, e);
                    }
                }
            }

            let now = Utc::now();
            let next = schedule.upcoming(Utc).next().unwrap();
            let duration = next - now;

            tokio::time::sleep(Duration::from_secs(duration.num_seconds() as u64)).await;
        }
    });
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
    conn: &mut DbConnection,
    id: Uuid,
    level_id: i32,
    two_player: bool,
) -> Result<(), ApiError> {
    let url = format!("https://gdladder.com/api/level/{}", level_id);

    let client = reqwest::Client::builder()
        .user_agent("AredlBackend/2.0 (+https://api.aredl.net)")
        .build()
        .map_err(|e| {
            ApiError::new(
                400,
                format!("Failed to build HTTP client: {:?}", e).as_str(),
            )
        })?;

    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|e| ApiError::new(400, &format!("Request failed: {:?}", e)))?
        .error_for_status()
        .map_err(|e| ApiError::new(400, &format!("HTTP error: {:?}", e)))?;

    let data: GDDLResponse = response
        .json()
        .await
        .map_err(|e| ApiError::new(400, format!("Failed to request gddl: {:?}", e).as_str()))?;

    let rating = match (two_player, data.two_player_rating, data.rating) {
        (true, Some(two_player_rating), _) => Some(two_player_rating),
        (true, None, _) => data.default_rating,
        (false, _, Some(rating)) => Some(rating),
        (false, _, None) => data.default_rating,
    };

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
    conn: &mut DbConnection,
    api_key: &String,
    spreadsheet_id: &String,
) -> Result<(), ApiError> {
    let ids_result = read_spreadsheet(api_key, spreadsheet_id, "'IDS'!B:D").await?;

    let data: Vec<(i32, f64, bool)> = ids_result
        .values
        .into_iter()
        .filter_map(|v| -> Option<(i32, f64, bool)> {
            if v.len() < 3 {
                return None;
            }
            let id = v[0].parse::<i32>().ok()?;
            let enjoyment = v[1].parse::<f64>().ok()?;
            let pending = v[2].parse::<bool>().unwrap_or(false);
            Some((id, enjoyment, pending))
        })
        .collect();

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
    conn: &mut DbConnection,
    api_key: &String,
    spreadsheet_id: &String,
) -> Result<(), ApiError> {
    let ids_result = read_spreadsheet(api_key, spreadsheet_id, "'IDS'!C:D").await?;

    let data: Vec<(i32, String)> = ids_result
        .values
        .into_iter()
        .filter_map(|v| -> Option<(i32, String)> {
            if v.len() < 2 {
                return None;
            }
            let id = v[0].parse::<i32>().ok()?;
            let tier = v[1].to_string();
            Some((id, tier))
        })
        .collect();

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
    api_key: &String,
    spreadsheet_id: &String,
    range: &str,
) -> Result<SheetValues, ApiError> {
    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}?key={}",
        spreadsheet_id, range, api_key
    );
    let response = reqwest::get(&url).await.map_err(|e| {
        ApiError::new(
            400,
            format!("Failed to request spreadsheet: {}", e).as_str(),
        )
    })?;
    if !response.status().is_success() {
        return Err(ApiError::new(400, "Failed to request spreadsheet"));
    }

    let sheet_values: SheetValues = response.json().await.map_err(|e| {
        ApiError::new(
            400,
            format!("Failed to request spreadsheet: {}", e).as_str(),
        )
    })?;

    Ok(sheet_values)
}
