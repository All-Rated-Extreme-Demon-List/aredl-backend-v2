#[cfg(test)]
use crate::db::DbConnection;
#[cfg(test)]
use crate::schema::aredl::records;
#[cfg(test)]
use diesel::{ExpressionMethods, RunQueryDsl};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_record(conn: &mut DbConnection, user_id: Uuid, level_id: Uuid) -> Uuid {
    diesel::insert_into(records::table)
        .values((
            records::submitted_by.eq(user_id),
            records::video_url.eq("https://example.com/video.mp4"),
            records::level_id.eq(level_id),
        ))
        .returning(records::id)
        .get_result::<Uuid>(conn)
        .expect("Failed to create test aredl record")
}
