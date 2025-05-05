use crate::{
    db::DbAppState,
    error_handler::ApiError,
    page_helper::{PageQuery, Paginated},
    schema::{aredl_shifts, users},
    users::BaseUser,
};
use chrono::{DateTime, Utc};
use diesel::{
    sql_types::Bool, AsChangeset, BoxableExpression, ExpressionMethods, Identifiable, IntoSql,
    JoinOnDsl, QueryDsl, Queryable, RunQueryDsl, SelectableHelper,
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
#[diesel(table_name = aredl_shifts)]
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
    pub user: BaseUser,
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

#[derive(Deserialize, ToSchema, AsChangeset)]
#[diesel(table_name = aredl_shifts)]
pub struct ShiftPatch {
    pub user_id: Option<Uuid>,
    pub status: Option<ShiftStatus>,
    pub completed_count: Option<i32>,
}

#[derive(Insertable)]
#[diesel(table_name = aredl_shifts)]
pub struct ShiftInsert {
    pub user_id: Uuid,
    pub target_count: i32,
    pub start_at: DateTime<Utc>,
    pub end_at: DateTime<Utc>,
}

impl Shift {
    pub fn patch(db: &DbAppState, id: Uuid, patch: ShiftPatch) -> Result<Self, ApiError> {
        let conn = &mut db.connection()?;

        let updated = diesel::update(aredl_shifts::table.filter(aredl_shifts::id.eq(id)))
            .set(&patch)
            .get_result::<Shift>(conn)?;
        Ok(updated)
    }

    pub fn delete(db: &DbAppState, id: Uuid) -> Result<Self, ApiError> {
        let conn = &mut db.connection()?;

        let deleted = diesel::delete(aredl_shifts::table.filter(aredl_shifts::id.eq(id)))
            .get_result::<Shift>(conn)?;
        Ok(deleted)
    }
}

impl ResolvedShift {
    pub fn from_data(shift_row: (Shift, BaseUser)) -> Self {
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
        db: &DbAppState,
        page_query: PageQuery<D>,
        user_id: Uuid,
    ) -> Result<Paginated<ShiftPage>, ApiError> {
        let conn = &mut db.connection()?;

        let total = aredl_shifts::table
            .filter(aredl_shifts::user_id.eq(user_id))
            .count()
            .get_result::<i64>(conn)?;

        let shift_rows = aredl_shifts::table
            .inner_join(users::table.on(aredl_shifts::user_id.eq(users::id)))
            .filter(aredl_shifts::user_id.eq(user_id))
            .order(aredl_shifts::start_at.desc())
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((Shift::as_select(), BaseUser::as_select()))
            .load::<(Shift, BaseUser)>(conn)?;

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
        db: &DbAppState,
        page_query: PageQuery<D>,
        options: ShiftFilterQuery,
    ) -> Result<Paginated<ShiftPage>, ApiError> {
        let conn = &mut db.connection()?;

        let total = aredl_shifts::table
            .into_boxed()
            .filter(options.user_id.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |user_id| Box::new(aredl_shifts::user_id.eq(user_id)),
            ))
            .filter(options.status.clone().map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |status| Box::new(aredl_shifts::status.eq(status)),
            ))
            .count()
            .get_result::<i64>(conn)?;

        let shift_rows = aredl_shifts::table
            .inner_join(users::table.on(aredl_shifts::user_id.eq(users::id)))
            .into_boxed()
            .filter(options.user_id.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |user_id| Box::new(aredl_shifts::user_id.eq(user_id)),
            ))
            .filter(options.status.map_or_else(
                || {
                    Box::new(true.into_sql::<Bool>())
                        as Box<dyn BoxableExpression<_, _, SqlType = Bool>>
                },
                |status| Box::new(aredl_shifts::status.eq(status)),
            ))
            .order(aredl_shifts::start_at.desc())
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select((Shift::as_select(), BaseUser::as_select()))
            .load::<(Shift, BaseUser)>(conn)?;

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
}
