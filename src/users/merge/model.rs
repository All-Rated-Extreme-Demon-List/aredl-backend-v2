use diesel::prelude::*;
use diesel::sql_types::Uuid as DieselUuid;
use diesel::{SelectableHelper, QueryDsl, RunQueryDsl};
use uuid::Uuid;
use chrono::NaiveDateTime;
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use diesel::pg::Pg;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::merge_logs;

#[derive(Serialize, Deserialize, Selectable, AsChangeset, Queryable, Debug, ToSchema)]
#[diesel(table_name = merge_logs, check_for_backend(Pg))]
pub struct MergeLog {
    /// Internal UUID of the log entry
    pub id: Uuid,
	/// Internal UUID of the primary user whose data was kept
	pub primary_user: Uuid,
	/// Internal UUID of the secondary user whose data was merged
	pub secondary_user: Uuid,
	/// Username of the secondary user before the merge
	pub secondary_username: String,
	/// Global name of the secondary user before the merge
	pub secondary_global_name: String,
	/// Discord ID of the secondary user before the merge
	pub secondary_discord_id: Option<String>,
	/// Timestamp of when the merge was executed
	pub merged_at: NaiveDateTime,
}

pub fn merge_users(
    conn: &mut DbConnection,
    primary_user: Uuid,
    secondary_user: Uuid,
) -> Result<(), ApiError> {

    diesel::sql_query("SELECT merge_users($1, $2);")
        .bind::<DieselUuid, _>(primary_user)
        .bind::<DieselUuid, _>(secondary_user)
        .execute(conn)?;
    Ok(())
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct MergeLogPage {
    pub data: Vec<MergeLog>
}

impl MergeLogPage {
	pub fn find_all<const D: i64>(conn: &mut DbConnection, page_query: PageQuery<D>) -> Result<Paginated<Self>, ApiError> {
		let data = merge_logs::table
			.select(MergeLog::as_select())
			.limit(page_query.per_page())
			.offset(page_query.offset())
			.order(merge_logs::merged_at.desc())
			.load::<MergeLog>(conn)?;

		let count: i64 = merge_logs::table.count().get_result::<i64>(conn)?;

		Ok(Paginated::<Self>::from_data(page_query, count, Self {
            data
        }))
	}
}
