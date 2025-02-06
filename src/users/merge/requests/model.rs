use diesel::prelude::*;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;
use chrono::NaiveDateTime;
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use diesel::pg::Pg;

use diesel::result::Error as DieselError;
use crate::db::DbConnection;

use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::users::merge::model::merge_users;
use crate::schema::merge_requests;

#[derive(Serialize, Deserialize, Debug, ToSchema, Insertable, AsChangeset)]
#[diesel(table_name = merge_requests, check_for_backend(Pg))]
pub struct MergeRequestUpsert {
	pub primary_user: Uuid,
	pub secondary_user: Uuid,
}

#[derive(Serialize, Deserialize, Selectable, AsChangeset, Queryable, Debug, ToSchema)]
#[diesel(table_name = merge_requests, check_for_backend(Pg))]
pub struct MergeRequest {
    /// Internal UUID of the merge request
    pub id: Uuid,
	/// Internal UUID of the primary user who made the request
	pub primary_user: Uuid,
	/// Internal UUID of the secondary user whose data will be merged
	pub secondary_user: Uuid,
	/// Whether the request was rejected or not (still pending)
	pub is_rejected: bool,
	/// Timestamp of when the request was made
	pub created_at: NaiveDateTime,
	/// Timestamp of when the request was last updated
	pub updated_at: NaiveDateTime,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct MergeRequestPage {
	pub data: Vec<MergeRequest>
}

impl MergeRequestPage {
	pub fn find_all<const D: i64>(conn: &mut DbConnection, page_query: PageQuery<D>) -> Result<Paginated<Self>, ApiError> {
		let data = merge_requests::table
			.select(MergeRequest::as_select())
			.limit(page_query.per_page())
			.offset(page_query.offset())
			.order(merge_requests::updated_at.desc())
			.load::<MergeRequest>(conn)?;

		let count: i64 = merge_requests::table.count().get_result::<i64>(conn)?;

		Ok(Paginated::<Self>::from_data(page_query, count, Self {
            data
        }))
	}
}

impl MergeRequest {
	pub fn upsert(conn: &mut DbConnection, request: MergeRequestUpsert) -> Result<Self, ApiError> {
		let new_request = diesel::insert_into(merge_requests::table)
            .values(&request)
            .on_conflict(merge_requests::primary_user)
			.filter_target(merge_requests::is_rejected.eq(true))
            .do_update()
            .set(&request)
            .returning(Self::as_select())
            .get_result::<Self>(conn);

        match new_request {
            Ok(merge_request) => Ok(merge_request),
            Err(DieselError::NotFound) => Err(ApiError::new(409,
                "You already submitted a merge request for your account. Please wait until it's either accepted or denied before submitting a new one.".into(),
            )),
            Err(e) => Err(ApiError::from(e)),
        }
	}

	pub fn accept(conn: &mut DbConnection, id: Uuid) -> Result<MergeRequest, ApiError> {
		
		let merge_request = merge_requests::table
			.filter(merge_requests::id.eq(id))
        	.get_result::<MergeRequest>(conn)?;

		merge_users(conn, merge_request.primary_user, merge_request.secondary_user)?;
		let result = diesel::delete(merge_requests::table)
			.filter(merge_requests::id.eq(id))
			.returning(Self::as_select())
			.get_result(conn)?;
		Ok(result)
	}

	pub fn reject(conn: &mut DbConnection, id: Uuid) -> Result<MergeRequest, ApiError> {

		let result = diesel::update(merge_requests::table)
            .set(merge_requests::is_rejected.eq(true))
            .filter(merge_requests::id.eq(id))
            .returning(Self::as_select())
            .get_result(conn)?;
        Ok(result)
	}
}
