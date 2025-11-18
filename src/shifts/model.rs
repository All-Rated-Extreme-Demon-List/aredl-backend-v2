use crate::{
    db::DbConnection,
    error_handler::ApiError,
    page_helper::{PageQuery, Paginated},
    schema::{shifts, users},
    users::BaseDiscordUser,
};
use chrono::{DateTime, Utc};
use diesel::{
    pg::Pg, AsChangeset, ExpressionMethods, Identifiable, JoinOnDsl, QueryDsl, Queryable,
    RunQueryDsl, SelectableHelper,
};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::ShiftStatus"]
#[DbValueStyle = "PascalCase"]
pub enum ShiftStatus {
    Running,
    Completed,
    Expired,
}

#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::Weekday"]
#[DbValueStyle = "PascalCase"]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

#[derive(
    Serialize, Deserialize, Debug, Selectable, Clone, Queryable, Identifiable, AsChangeset, ToSchema,
)]
#[diesel(table_name = shifts)]
pub struct Shift {
    /// Internal UUID of the shift.
    pub id: Uuid,
    /// UUID of the user this shift is assigned to.
    pub user_id: Uuid,
    /// The target number of submissions to review for this shift.
    pub target_count: i32,
    /// The number of submissions that have been reviewed for this shift.
    pub completed_count: i32,
    /// The start time of the shift.
    pub start_at: DateTime<Utc>,
    /// The end time of the shift.
    pub end_at: DateTime<Utc>,
    /// The current status of the shift.
    pub status: ShiftStatus,
    /// Timestamp of when the shift was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the shift was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ResolvedShift {
    /// Internal UUID of the shift.
    pub id: Uuid,
    /// User this shift is assigned to.
    pub user: BaseDiscordUser,
    /// The target number of submissions to review for this shift.
    pub target_count: i32,
    /// The number of submissions that have been reviewed for this shift.
    pub completed_count: i32,
    /// The start time of the shift.
    pub start_at: DateTime<Utc>,
    /// The end time of the shift.
    pub end_at: DateTime<Utc>,
    /// The current status of the shift.
    pub status: ShiftStatus,
    /// Timestamp of when the shift was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the shift was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Deserialize, Debug, ToSchema)]
pub struct ShiftFilterQuery {
    pub user_id: Option<Uuid>,
    pub status: Option<ShiftStatus>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ShiftPage {
    pub data: Vec<ResolvedShift>,
}

#[derive(Deserialize, ToSchema, AsChangeset, Debug)]
#[diesel(table_name = shifts)]
pub struct ShiftPatch {
    pub user_id: Option<Uuid>,
    pub status: Option<ShiftStatus>,
    pub completed_count: Option<i32>,
}

#[derive(Insertable, Serialize, Clone)]
#[diesel(table_name = shifts)]
pub struct ShiftInsert {
    pub user_id: Uuid,
    pub target_count: i32,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

impl Shift {
    pub fn patch(conn: &mut DbConnection, id: Uuid, patch: ShiftPatch) -> Result<Self, ApiError> {
        let updated = diesel::update(shifts::table.filter(shifts::id.eq(id)))
            .set(&patch)
            .get_result::<Shift>(conn)?;
        Ok(updated)
    }

    pub fn delete(conn: &mut DbConnection, id: Uuid) -> Result<Self, ApiError> {
        let deleted =
            diesel::delete(shifts::table.filter(shifts::id.eq(id))).get_result::<Shift>(conn)?;
        Ok(deleted)
    }
}

impl ResolvedShift {
    pub fn from_data(shift_row: (Shift, BaseDiscordUser)) -> Self {
        let (shift, user) = shift_row;
        Self {
            id: shift.id,
            user,
            target_count: shift.target_count,
            completed_count: shift.completed_count,
            start_at: shift.start_at,
            end_at: shift.end_at,
            status: shift.status,
            created_at: shift.created_at,
            updated_at: shift.updated_at,
        }
    }
}

impl ShiftPage {
    pub fn find_me<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        user_id: Uuid,
    ) -> Result<Paginated<ShiftPage>, ApiError> {
        let total = shifts::table
            .filter(shifts::user_id.eq(user_id))
            .count()
            .get_result::<i64>(conn)?;

        let shift_rows = shifts::table
            .inner_join(users::table.on(shifts::user_id.eq(users::id)))
            .filter(shifts::user_id.eq(user_id))
            .order(shifts::start_at.desc())
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((Shift::as_select(), BaseDiscordUser::as_select()))
            .load::<(Shift, BaseDiscordUser)>(conn)?;

        let resolved_shifts = shift_rows
            .into_iter()
            .map(ResolvedShift::from_data)
            .collect::<Vec<_>>();

        Ok(Paginated::from_data(
            page_query,
            total,
            ShiftPage {
                data: resolved_shifts,
            },
        ))
    }

    pub fn find_all<const D: i64>(
        conn: &mut DbConnection,
        page_query: PageQuery<D>,
        options: ShiftFilterQuery,
    ) -> Result<Paginated<ShiftPage>, ApiError> {
        let build_filtered = || {
            let mut q = shifts::table.into_boxed::<Pg>();
            if let Some(user_id) = options.user_id {
                q = q.filter(shifts::user_id.eq(user_id));
            }
            if let Some(status) = options.status.clone() {
                q = q.filter(shifts::status.eq(status));
            }
            q
        };

        let total_count: i64 = build_filtered().count().get_result(conn)?;

        let shift_rows: Vec<(Shift, BaseDiscordUser)> = build_filtered()
            .inner_join(users::table.on(shifts::user_id.eq(users::id)))
            .order(shifts::start_at.desc())
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((Shift::as_select(), BaseDiscordUser::as_select()))
            .load(conn)?;

        let resolved_shifts = shift_rows
            .into_iter()
            .map(ResolvedShift::from_data)
            .collect::<Vec<_>>();

        Ok(Paginated::from_data(
            page_query,
            total_count,
            ShiftPage {
                data: resolved_shifts,
            },
        ))
    }
}
