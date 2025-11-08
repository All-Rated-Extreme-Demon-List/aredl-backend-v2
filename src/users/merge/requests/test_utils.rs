#[cfg(test)]
use crate::{db::DbConnection, schema::merge_requests};
#[cfg(test)]
use diesel::{ExpressionMethods, RunQueryDsl};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_merge_req(conn: &mut DbConnection, user_1: Uuid, user_2_id: Uuid) -> Uuid {
    diesel::insert_into(merge_requests::table)
        .values((
            // this becomes the new user
            merge_requests::primary_user.eq(user_1),
            merge_requests::secondary_user.eq(user_2_id),
            merge_requests::is_rejected.eq(false),
            merge_requests::is_claimed.eq(false),
        ))
        .returning(merge_requests::id)
        .get_result::<Uuid>(conn)
        .expect("Failed to create test merge request!")
}
