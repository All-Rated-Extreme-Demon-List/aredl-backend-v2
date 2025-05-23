use crate::{
    arepl::shifts::{ShiftInsert, Weekday},
    db::DbAppState,
    error_handler::ApiError,
    schema::{
        arepl::{recurrent_shifts, shifts},
        users,
    },
    users::BaseUser,
};
use chrono::{DateTime, Datelike, NaiveDate, TimeZone, Utc};
use diesel::{
    AsChangeset, ExpressionMethods, Identifiable, Insertable, JoinOnDsl, PgConnection, QueryDsl,
    Queryable, RunQueryDsl, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(
    Serialize, Deserialize, Selectable, Debug, Clone, Queryable, Identifiable, AsChangeset, ToSchema,
)]
#[diesel(table_name = recurrent_shifts)]
pub struct RecurringShift {
    /// Internal UUID of the regular shift.
    pub id: Uuid,
    /// UUID of the user this shift is regularly assigned to.
    pub user_id: Uuid,
    /// The day of the week this shift is assigned at.
    pub weekday: Weekday,
    /// The start time of the shift on the assigned day, in hour compared to UTC
    pub start_hour: i32,
    /// How long this shift should last
    pub duration: i32,
    /// The target number of submissions to review for this shift.
    pub target_count: i32,
    /// The timestamp of when this regular shift was created.
    pub created_at: DateTime<Utc>,
    /// The timestamp of when this regular shift was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct ResolvedRecurringShift {
    /// Internal UUID of the regular shift.
    pub id: Uuid,
    /// UUID of the user this shift is regularly assigned to.
    pub user: BaseUser,
    /// The day of the week this shift is assigned at.
    pub weekday: Weekday,
    /// The start time of the shift on the assigned day, in hour compared to UTC
    pub start_hour: i32,
    /// How long this shift should last
    pub duration: i32,
    /// The target number of submissions to review for this shift.
    pub target_count: i32,
    /// The timestamp of when this regular shift was created.
    pub created_at: DateTime<Utc>,
    /// The timestamp of when this regular shift was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Insertable, ToSchema)]
#[diesel(table_name = recurrent_shifts)]
pub struct RecurringShiftInsert {
    /// UUID of the user to assign a regular shift to.
    pub user_id: Uuid,
    /// The day of the week this shift is assigned at.
    pub weekday: Weekday,
    /// The start time of the shift on the assigned day.
    pub start_hour: i32,
    /// How long this shift should last
    pub duration: i32,
    /// The target number of submissions to review for this shift.
    pub target_count: i32,
}

#[derive(Deserialize, ToSchema, AsChangeset, Debug)]
#[diesel(table_name = recurrent_shifts)]
pub struct RecurringShiftPatch {
    pub user_id: Option<Uuid>,
    pub weekday: Option<Weekday>,
    pub target_count: Option<i32>,
    pub start_hour: Option<i32>,
    pub duration: Option<i32>,
}

impl ResolvedRecurringShift {
    pub fn from_data(recurring_shift_row: (RecurringShift, BaseUser)) -> Self {
        let (recurring_shift, user) = recurring_shift_row;
        Self {
            id: recurring_shift.id,
            user,
            weekday: recurring_shift.weekday,
            start_hour: recurring_shift.start_hour,
            duration: recurring_shift.duration,
            target_count: recurring_shift.target_count,
            created_at: recurring_shift.created_at,
            updated_at: recurring_shift.updated_at,
        }
    }

    pub fn find_all(db: &DbAppState) -> Result<Vec<Self>, ApiError> {
        let conn = &mut db.connection()?;

        let result_rows = recurrent_shifts::table
            .inner_join(users::table.on(recurrent_shifts::user_id.eq(users::id)))
            .order((
                recurrent_shifts::weekday.asc(),
                recurrent_shifts::start_hour.asc(),
            ))
            .select((RecurringShift::as_select(), BaseUser::as_select()))
            .load::<(RecurringShift, BaseUser)>(conn)?;

        let result = result_rows
            .into_iter()
            .map(ResolvedRecurringShift::from_data)
            .collect::<Vec<_>>();

        Ok(result)
    }
}

impl RecurringShift {
    pub fn create(db: &DbAppState, new_shift: RecurringShiftInsert) -> Result<Self, ApiError> {
        let conn = &mut db.connection()?;
        let inserted = diesel::insert_into(recurrent_shifts::table)
            .values(&new_shift)
            .get_result(conn)?;
        Ok(inserted)
    }

    pub fn patch(db: &DbAppState, id: Uuid, patch: RecurringShiftPatch) -> Result<Self, ApiError> {
        let conn = &mut db.connection()?;

        let updated = diesel::update(recurrent_shifts::table.filter(recurrent_shifts::id.eq(id)))
            .set(&patch)
            .get_result::<RecurringShift>(conn)?;
        Ok(updated)
    }

    pub fn delete(db: &DbAppState, id: Uuid) -> Result<Self, ApiError> {
        let conn = &mut db.connection()?;

        let deleted = diesel::delete(recurrent_shifts::table.filter(recurrent_shifts::id.eq(id)))
            .get_result::<RecurringShift>(conn)?;
        Ok(deleted)
    }

    pub fn create_shifts(conn: &mut PgConnection, date: NaiveDate) -> Result<(), ApiError> {
        let today = match date.weekday().number_from_monday() {
            1 => Weekday::Monday,
            2 => Weekday::Tuesday,
            3 => Weekday::Wednesday,
            4 => Weekday::Thursday,
            5 => Weekday::Friday,
            6 => Weekday::Saturday,
            7 => Weekday::Sunday,
            _ => unreachable!(),
        };

        let templates: Vec<RecurringShift> = recurrent_shifts::table
            .filter(recurrent_shifts::weekday.eq(today))
            .load(conn)?;

        for template in templates {
            let naive_dt = date
                .and_hms_opt(template.start_hour as u32, 0, 0)
                .ok_or_else(|| ApiError::new(400, "invalid start_hour".into()))?;
            let start_at: DateTime<Utc> = Utc.from_utc_datetime(&naive_dt);

            let end_at = start_at + chrono::Duration::hours(template.duration as i64);

            let exists: i64 = shifts::table
                .filter(shifts::user_id.eq(template.user_id))
                .filter(shifts::start_at.eq(start_at))
                .count()
                .get_result(conn)?;

            if exists == 0 {
                let new = ShiftInsert {
                    user_id: template.user_id,
                    target_count: template.target_count,
                    start_at,
                    end_at,
                };
                diesel::insert_into(shifts::table)
                    .values(&new)
                    .execute(conn)?;
            }
        }

        Ok(())
    }
}
