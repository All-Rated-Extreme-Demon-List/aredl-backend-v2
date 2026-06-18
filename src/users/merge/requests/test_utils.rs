#[cfg(test)]
use std::sync::Arc;

#[cfg(test)]
use crate::{
    app_data::db::DbAppState, schema::merge_requests, users::merge::requests::MergeRequest,
};
#[cfg(test)]
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
#[cfg(test)]
use uuid::Uuid;

#[cfg(test)]
pub async fn create_test_merge_req(db: &Arc<DbAppState>, user_1: Uuid, user_2_id: Uuid) -> Uuid {
    diesel::insert_into(merge_requests::table)
        .values((
            // this becomes the new user
            merge_requests::primary_user.eq(user_1),
            merge_requests::secondary_user.eq(user_2_id),
            merge_requests::is_rejected.eq(false),
            merge_requests::is_claimed.eq(false),
        ))
        .returning(merge_requests::id)
        .get_result::<Uuid>(&mut db.connection().unwrap())
        .expect("Failed to create test merge request!")
}

#[cfg(test)]
pub fn set_test_merge_request_claimed(db: &Arc<DbAppState>, merge_id: Uuid, is_claimed: bool) {
    diesel::update(merge_requests::table)
        .filter(merge_requests::id.eq(merge_id))
        .set(merge_requests::is_claimed.eq(is_claimed))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set test merge request claimed flag");
}

#[cfg(test)]
pub fn set_test_merge_request_rejected(db: &Arc<DbAppState>, merge_id: Uuid, is_rejected: bool) {
    diesel::update(merge_requests::table)
        .filter(merge_requests::id.eq(merge_id))
        .set(merge_requests::is_rejected.eq(is_rejected))
        .execute(&mut db.connection().unwrap())
        .expect("Failed to set test merge request rejected flag");
}

#[cfg(test)]
pub fn get_test_merge_request(db: &Arc<DbAppState>, merge_id: Uuid) -> MergeRequest {
    get_test_merge_request_optional(db, merge_id).expect("Failed to fetch test merge request")
}

#[cfg(test)]
pub fn get_test_merge_request_optional(
    db: &Arc<DbAppState>,
    merge_id: Uuid,
) -> Option<MergeRequest> {
    merge_requests::table
        .filter(merge_requests::id.eq(merge_id))
        .select(MergeRequest::as_select())
        .first(&mut db.connection().unwrap())
        .optional()
        .expect("Failed to fetch test merge request")
}
