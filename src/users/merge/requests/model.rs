use chrono::{DateTime, Utc};
use diesel::dsl::now;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::page_helper::{PageQuery, Paginated};
use crate::schema::{merge_requests, users};
use crate::users::me::notifications::{Notification, NotificationType};
use crate::users::merge::merge_users;
use crate::users::BaseUser;

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
    /// Whether the request was claimed and under review or not
    pub is_claimed: bool,
    /// Timestamp of when the request was made
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the request was last updated
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ResolvedMergeRequest {
    /// Internal UUID of the merge request
    pub id: Uuid,
    /// Primary user who made the request
    pub primary_user: BaseUser,
    /// Secondary user whose data will be merged
    pub secondary_user: BaseUser,
    /// Whether the request was rejected or not (still pending)
    pub is_rejected: bool,
    /// Whether the request was claimed and under review or not
    pub is_claimed: bool,
    /// Timestamp of when the request was made
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the request was last updated
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct MergeRequestPage {
    pub data: Vec<ResolvedMergeRequest>,
}

impl ResolvedMergeRequest {
    pub fn from_data(row: (MergeRequest, BaseUser, BaseUser)) -> Self {
        Self {
            id: row.0.id,
            primary_user: row.1,
            secondary_user: row.2,
            is_rejected: row.0.is_rejected,
            is_claimed: row.0.is_claimed,
            created_at: row.0.created_at,
            updated_at: row.0.updated_at,
        }
    }
}

impl MergeRequestPage {
    pub fn find_all<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
    ) -> Result<Paginated<Self>, ApiError> {
        let users2 = alias!(users as users2);
        let data_rows = merge_requests::table
            .inner_join(users::table.on(merge_requests::primary_user.eq(users::id)))
            .inner_join(users2.on(merge_requests::secondary_user.eq(users2.field(users::id))))
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((
                MergeRequest::as_select(),
                BaseUser::as_select(),
                users2.fields(<BaseUser as Selectable<Pg>>::construct_selection()),
            ))
            .order(merge_requests::updated_at.desc())
            .load::<(MergeRequest, BaseUser, BaseUser)>(conn)?;

        let count: i64 = merge_requests::table.count().get_result::<i64>(conn)?;

        let data = data_rows
            .into_iter()
            .map(|row| ResolvedMergeRequest::from_data(row))
            .collect::<Vec<_>>();

        Ok(Paginated::<Self>::from_data(
            page_query,
            count,
            Self { data },
        ))
    }
}

impl MergeRequest {
    pub fn upsert(conn: &mut DbConnection, request: MergeRequestUpsert) -> Result<Self, ApiError> {
        if request.primary_user == request.secondary_user {
            return Err(ApiError::new(
                400,
                "You cannot merge your account with itself.".into(),
            ));
        }

        let secondary_user_data = users::table
            .filter(users::id.eq(request.secondary_user))
            .select((users::id, users::placeholder))
            .first::<(Uuid, bool)>(conn)
            .optional()?;

        if let Some((_user_id, is_placeholder)) = secondary_user_data {
            if !is_placeholder {
                return Err(ApiError::new(400, "You can only submit merge requests for placeholder users. To merge your account with a user that is already linked to another discord account, please make a support post on our discord server.".into()));
            }
        } else {
            return Err(ApiError::new(
                404,
                "The secondary user does not exist.".into(),
            ));
        }

        let existing_request = merge_requests::table
            .filter(merge_requests::primary_user.eq(request.primary_user))
            .select(MergeRequest::as_select())
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

    pub fn claim(conn: &mut DbConnection) -> Result<Option<Self>, ApiError> {
        let next_id = merge_requests::table
            .filter(merge_requests::is_claimed.eq(false))
            .filter(merge_requests::is_rejected.eq(false))
            .select(merge_requests::id)
            .order(merge_requests::updated_at.asc())
            .for_update()
            .skip_locked()
            .first::<Uuid>(conn)
            .optional()?;

        if next_id.is_none() {
            return Ok(None);
        }

        let result = diesel::update(merge_requests::table)
            .set(merge_requests::is_claimed.eq(true))
            .filter(merge_requests::id.eq(next_id.unwrap()))
            .returning(Self::as_select())
            .get_result::<Self>(conn)
            .optional()?;
        Ok(result)
    }

    pub fn unclaim(conn: &mut DbConnection, id: Uuid) -> Result<Self, ApiError> {
        let result = diesel::update(merge_requests::table)
            .set(merge_requests::is_claimed.eq(false))
            .filter(merge_requests::id.eq(id))
            .returning(Self::as_select())
            .get_result::<Self>(conn)?;
        Ok(result)
    }

    pub fn accept(conn: &mut DbConnection, id: Uuid) -> Result<(), ApiError> {
        let merge_request = merge_requests::table
            .filter(merge_requests::id.eq(id))
            .select(MergeRequest::as_select())
            .get_result::<MergeRequest>(conn)?;
        merge_users(
            conn,
            merge_request.primary_user,
            merge_request.secondary_user,
        )?;

        Notification::create(
            conn,
            merge_request.primary_user,
            "Your merge request has been accepted!".to_string(),
            NotificationType::Success,
        )?;

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

        Notification::create(
            conn,
            result.primary_user,
            "Your merge request has been rejected.".to_string(),
            NotificationType::Failure,
        )?;
        Ok(result)
    }
}
