pub mod data_cleaner;
pub mod refresh_discord_avatars;
pub mod refresh_level_data;
pub mod refresh_matviews;
pub mod shifts_creator;
pub mod sync_patreon_plus;

use crate::error_handler::{ConfigError, StartupError};
use crate::get_secret;
use chrono::Utc;
use cron::Schedule;
use std::str::FromStr as _;
use std::sync::Arc;
use std::time::Duration;

#[cfg(test)]
mod tests;

pub fn startup_schedule(var_name: &'static str) -> Result<Arc<Schedule>, StartupError> {
    let value = get_secret(var_name)?;
    parse_startup_schedule(var_name, &value)
}

pub fn parse_startup_schedule(
    name: &'static str,
    value: &str,
) -> Result<Arc<Schedule>, StartupError> {
    Schedule::from_str(value).map(Arc::new).map_err(|error| {
        ConfigError::InvalidValue {
            name: name.to_owned(),
            message: error.to_string(),
        }
        .into()
    })
}

pub async fn sleep_until_next(schedule: &Schedule) {
    let now = Utc::now();
    let Some(next) = schedule.upcoming(Utc).next() else {
        tracing::error!("Schedule has no upcoming execution time");
        tokio::time::sleep(Duration::from_mins(1)).await;
        return;
    };

    tokio::time::sleep((next - now).to_std().unwrap_or_default()).await;
}
