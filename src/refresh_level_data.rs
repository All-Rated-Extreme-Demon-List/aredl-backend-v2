use std::env;
use std::str::FromStr;
use std::sync::Arc;
use std::time::Duration;
use chrono::Utc;
use cron::Schedule;
use diesel::{BoolExpressionMethods, Connection, ExpressionMethods, JoinOnDsl, QueryDsl, QueryResult, RunQueryDsl, select};
use diesel::dsl::exists;
use serde::Deserialize;
use tokio::task;
use uuid::Uuid;
use crate::db::{DbAppState, DbConnection};
use crate::error_handler::ApiError;
use crate::schema::{aredl_last_gddl_update, aredl_levels};

pub async fn start_level_data_refresher(db: Arc<DbAppState>) {
    let schedule = Schedule::from_str(env::var("LEVEL_DATA_REFRESH_SCHEDULE")
        .expect("LEVEL_DATA_REFRESH_SCHEDULE not set").as_str()).unwrap();
    let schedule = Arc::new(schedule);

    let google_api_key = env::var("GOOGLE_API_KEY")
        .expect("GOOGLE_API_KEY not set");
    let edel_sheet_id = env::var("EDEL_SHEET_ID")
        .expect("EDEL_SHEET_ID not set");
    let nlw_sheet_id = env::var("NLW_SHEET_ID")
        .expect("NLW_SHEET_ID not set");

    let db_clone = db.clone();
    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            println!("Refreshing level data");

            let conn = db_clone.connection();

            if conn.is_err() {
                println!("Failed to refresh {}", conn.err().unwrap());
                continue;
            }

            let mut conn = conn.unwrap();

            let edel_result = update_edel_data(&mut conn, &google_api_key, &edel_sheet_id).await;

            if edel_result.is_err() {
                println!("Failed to refresh edel {}", edel_result.err().unwrap());
            }

            let nlw_result = update_nlw_data(&mut conn, &google_api_key, &nlw_sheet_id).await;

            if nlw_result.is_err() {
                println!("Failed to refresh nlw {}", nlw_result.err().unwrap());
            }

            let now = Utc::now();
            let next = schedule.upcoming(Utc).next().unwrap();
            let duration = next - now;

            tokio::time::sleep(Duration::from_secs(duration.num_seconds() as u64)).await;
        }
    });

    let schedule = Schedule::from_str("@hourly")
        .expect("Failed to parse schedule");
    let schedule = Arc::new(schedule);

    task::spawn(async move {
        loop {
            tokio::time::sleep(Duration::from_secs(5)).await;

            println!("Running gddl updater");

            let conn = db.connection();

            if conn.is_err() {
                println!("Failed to refresh {}", conn.err().unwrap());
                continue;
            }

            let mut conn = conn.unwrap();

            let one_day_ago = Utc::now().naive_utc() - chrono::Duration::days(1);

            let to_update: QueryResult<Vec<(Uuid, i32, bool)>> = aredl_levels::table
                .left_join(aredl_last_gddl_update::table
                    .on(aredl_last_gddl_update::id.eq(aredl_levels::id)))
                .filter(
                    aredl_last_gddl_update::updated_at.is_null().or(
                        aredl_last_gddl_update::updated_at.lt(one_day_ago)
                    )
                )
                .select((
                    aredl_levels::id,
                    aredl_levels::level_id,
                    aredl_levels::two_player
                    ))
                .load::<(Uuid, i32, bool)>(&mut conn);

            match to_update {
                Ok(to_update) => {
                    for (id, level_id, two_player) in to_update {
                        tokio::time::sleep(Duration::from_secs(5)).await;
                        let result = update_gddl_data(&mut conn, id, level_id, two_player).await;
                        if result.is_err() {
                            println!("Failed to request gddl: {}, {}", level_id, result.err().unwrap());
                        } else {
                            //println!("Updated gddl data for {}, {}", id, level_id)
                        }
                    }
                },
                Err(e) => {
                    println!("Failed to load gddl update db: {}", e);
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
    #[serde(rename = "TwoPlayerRating")]
    two_player_rating: Option<f64>
}

async fn update_gddl_data(conn: &mut DbConnection, id: Uuid, level_id: i32, two_player: bool) -> Result<(), ApiError> {
    let url = format!(
        "https://gdladder.com/api/level/{}",
        level_id
    );
    let response = reqwest::get(&url).await
        .map_err(|e| ApiError::new(400, format!("Failed to request gddl: {}", e).as_str()))?;
    let data: GDDLResponse = response.json().await
        .map_err(|e| ApiError::new(400, format!("Failed to request gddl: {}", e).as_str()))?;

    let rating = match (two_player, data.two_player_rating) {
        (false, _) => data.rating,
        (true, rating) => rating
    };

    diesel::update(aredl_levels::table)
        .filter(aredl_levels::id.eq(id))
        .set(aredl_levels::gddl_tier.eq(rating))
        .execute(conn)?;

    diesel::insert_into(aredl_last_gddl_update::table)
        .values((
            aredl_last_gddl_update::id.eq(id),
            aredl_last_gddl_update::updated_at.eq(Utc::now().naive_utc())
            ))
        .on_conflict(aredl_last_gddl_update::id)
        .do_update()
        .set(aredl_last_gddl_update::updated_at.eq(Utc::now().naive_utc()))
        .execute(conn)?;

    Ok(())
}

async fn update_edel_data(conn: &mut DbConnection, api_key: &String, spreadsheet_id: &String) -> Result<(), ApiError> {

    let ids_result = read_spreadsheet(api_key, spreadsheet_id, "'IDS'!B:D").await?;

    let data: Vec<(i32, f64, bool)> = ids_result.values
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
        for (id, enjoyment, pending) in data {
            let exists_2p = select(
                exists(aredl_levels::table
                    .filter(aredl_levels::level_id.eq(id))
                    .filter(aredl_levels::two_player.eq(true))
                )
            ).get_result::<bool>(conn)?;

            diesel::update(aredl_levels::table)
                .set((
                    aredl_levels::edel_enjoyment.eq(enjoyment),
                    aredl_levels::is_edel_pending.eq(pending)
                    ))
                .filter(aredl_levels::level_id.eq(id))
                .filter(aredl_levels::two_player.eq(exists_2p))
                .execute(conn)?;
        }
        Ok(())
    })
}

async fn update_nlw_data(conn: &mut DbConnection, api_key: &String, spreadsheet_id: &String) -> Result<(), ApiError> {

    let ids_result = read_spreadsheet(api_key, spreadsheet_id, "'IDS'!C:D").await?;

    let data: Vec<(i32, String)> = ids_result.values
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
        for (id, tier) in data {
            let exists_2p = select(
                exists(aredl_levels::table
                    .filter(aredl_levels::level_id.eq(id))
                    .filter(aredl_levels::two_player.eq(true))
                )
            ).get_result::<bool>(conn)?;

            diesel::update(aredl_levels::table)
                .set((
                    aredl_levels::nlw_tier.eq(tier),
                ))
                .filter(aredl_levels::level_id.eq(id))
                .filter(aredl_levels::two_player.eq(exists_2p))
                .execute(conn)?;
        }
        Ok(())
    })
}

#[derive(Deserialize)]
struct SheetValues {
    values: Vec<Vec<String>>,
}

async fn read_spreadsheet(api_key: &String, spreadsheet_id: &String, range: &str) -> Result<SheetValues, ApiError> {
    let url = format!(
        "https://sheets.googleapis.com/v4/spreadsheets/{}/values/{}?key={}",
        spreadsheet_id, range, api_key
    );
    let response = reqwest::get(&url)
        .await
        .map_err(|e| ApiError::new(400, format!("Failed to request spreadsheet: {}", e).as_str()))?;
    if !response.status().is_success() {
        return Err(ApiError::new(400, "Failed to request spreadsheet"))
    }

    let sheet_values: SheetValues = response.json().await
        .map_err(|e| ApiError::new(400, format!("Failed to request spreadsheet: {}", e).as_str()))?;

    Ok(sheet_values)
}