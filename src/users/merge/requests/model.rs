use diesel::prelude::*;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use diesel::dsl::now;
use uuid::Uuid;
use chrono::NaiveDateTime;
use utoipa::ToSchema;
use serde::{Deserialize, Serialize};
use diesel::pg::Pg;

use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::users::merge::model::merge_users;
use crate::schema::{users, merge_requests};

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

		if request.primary_user == request.secondary_user {
            return Err(ApiError::new(400, "You cannot merge your account with itself.".into()));
        }

        let existing_secondary = users::table
            .filter(users::id.eq(request.secondary_user))
            .select(users::id)
            .first::<Uuid>(conn)
            .optional()?;

        if existing_secondary.is_none() {
            return Err(ApiError::new(404, "The secondary user does not exist.".into()));
        }

		let existing_request = merge_requests::table
			.filter(merge_requests::primary_user.eq(request.primary_user))
			.first::<MergeRequest>(conn)
			.optional()?;
        
		if let Some(existing) = existing_request {
			if !existing.is_rejected {
				return Err(ApiError::new(409,
					"You already submitted a merge request for your account. Please wait until it's either accepted or denied before submitting a new one.".into(),
				));
			}
		}

		let changes = (
			&request,
			merge_requests::is_rejected.eq(false),
			merge_requests::updated_at.eq(now),
		);

		let new_request = diesel::insert_into(merge_requests::table)
            .values(&request)
            .on_conflict(merge_requests::primary_user)
            .do_update()
            .set(changes)
            .returning(Self::as_select())
            .get_result::<Self>(conn)?;

		Ok(new_request)
	}

	pub fn accept(conn: &mut DbConnection, id: Uuid) -> Result<(), ApiError> {
		let merge_request = merge_requests::table
			.filter(merge_requests::id.eq(id))
        	.get_result::<MergeRequest>(conn)?;
		merge_users(conn, merge_request.primary_user, merge_request.secondary_user)?;
		diesel::delete(merge_requests::table)
			.filter(merge_requests::id.eq(id))
			.execute(conn)?;
		Ok(())
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
