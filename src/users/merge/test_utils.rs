#[cfg(test)]
use crate::{db::DbConnection, schema::merge_logs};
use chrono::Utc;
#[cfg(test)]
use diesel::{ExpressionMethods, RunQueryDsl};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_merge_log(
    conn: &mut DbConnection,
    primary_user: Uuid,
    secondary_user: Uuid,
) -> Uuid {
    diesel::insert_into(merge_logs::table)
        .values((
            merge_logs::primary_user.eq(primary_user),
            merge_logs::secondary_user.eq(secondary_user),
            merge_logs::secondary_username.eq("Placeholder"),
            merge_logs::secondary_global_name.eq("Placeholder"),
            merge_logs::secondary_discord_id.eq(None::<String>),
            merge_logs::merged_at.eq(Utc::now()),
        ))
        .returning(merge_logs::id)
        .get_result::<Uuid>(conn)
        .expect("Failed to create test merge log")
}
