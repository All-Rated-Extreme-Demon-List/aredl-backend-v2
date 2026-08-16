use crate::{
    app_data::db::DbConnection,
    auth::Authenticated,
    error_handler::ApiError,
    roles::ReviewerVisibility,
    schema::{recurrent_shifts, shifts, users},
    shifts::{ShiftInsert, Weekday},
    users::BaseUser,
};
use chrono::{DateTime, NaiveDate, TimeZone as _, Utc};
use chrono_tz::Tz;
use diesel::{pg::Pg, AsChangeset, Identifiable, Insertable, Queryable};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use diesel::prelude::*;
#[derive(
    Serialize, Deserialize, Selectable, Debug, Clone, Queryable, Identifiable, AsChangeset, ToSchema,
)]
#[diesel(table_name = recurrent_shifts, check_for_backend(Pg))]
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
    /// The timezone this shift is in, as an IANA timezone string (e.g., "America/New_York").
    pub timezone: String,
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
    /// The timezone this shift is in, as an IANA timezone string (e.g., "America/New_York").
    pub timezone: String,
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
    /// The timezone this shift is in, as an IANA timezone string (e.g., "America/New_York").
    pub timezone: String,
    /// The target number of submissions to review for this shift.
    pub target_count: i32,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct SelfRecurringShiftInsert {
    /// The day of the week this shift is assigned at.
    pub weekday: Weekday,
    /// The start time of the shift on the assigned day.
    pub start_hour: i32,
    /// The timezone this user is in, as an IANA timezone string (e.g., "America/New_York").
    pub timezone: String,
    /// How long this shift should last, in hours.
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
    pub timezone: Option<String>,
}

pub fn parse_timezone(timezone: &str) -> Result<Tz, ApiError> {
    timezone.parse::<Tz>().map_err(|_err| {
        ApiError::BadRequest(
            "Invalid timezone provided. Please provide a valid IANA timezone string.",
        )
    })
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
            timezone: recurring_shift.timezone,
            target_count: recurring_shift.target_count,
            created_at: recurring_shift.created_at,
            updated_at: recurring_shift.updated_at,
        }
    }

    pub fn find_all_for_user(
        conn: &mut DbConnection,
        authenticated: &Authenticated,
    ) -> Result<Vec<Self>, ApiError> {
        let result_rows = recurrent_shifts::table
            .inner_join(users::table.on(recurrent_shifts::user_id.eq(users::id)))
            .order((
                recurrent_shifts::weekday.asc(),
                recurrent_shifts::start_hour.asc(),
            ))
            .select((RecurringShift::as_select(), BaseUser::as_select()))
            .load::<(RecurringShift, BaseUser)>(conn)?;

        let mut result = result_rows
            .into_iter()
            .map(ResolvedRecurringShift::from_data)
            .collect::<Vec<_>>();

        let visibility = ReviewerVisibility::new(conn, authenticated)?;

        result.retain(|shift| visibility.can_see_identity(&shift.user.id));

        Ok(result)
    }
}

impl RecurringShift {
    pub fn create(
        conn: &mut DbConnection,
        new_shift: &RecurringShiftInsert,
    ) -> Result<Self, ApiError> {
        let inserted = diesel::insert_into(recurrent_shifts::table)
            .values(new_shift)
            .returning(RecurringShift::as_select())
            .get_result(conn)?;
        Ok(inserted)
    }

    pub fn patch(
        conn: &mut DbConnection,
        id: Uuid,
        patch: &RecurringShiftPatch,
    ) -> Result<Self, ApiError> {
        let updated = diesel::update(recurrent_shifts::table.filter(recurrent_shifts::id.eq(id)))
            .set(patch)
            .returning(RecurringShift::as_select())
            .get_result::<RecurringShift>(conn)?;
        Ok(updated)
    }

    pub fn delete(conn: &mut DbConnection, id: Uuid) -> Result<Self, ApiError> {
        let deleted = diesel::delete(recurrent_shifts::table.filter(recurrent_shifts::id.eq(id)))
            .returning(RecurringShift::as_select())
            .get_result::<RecurringShift>(conn)?;
        Ok(deleted)
    }

    pub fn create_shifts(
        conn: &mut DbConnection,
        date: NaiveDate,
    ) -> Result<Vec<ShiftInsert>, ApiError> {
        let today = Weekday::from(date);

        let templates: Vec<RecurringShift> = recurrent_shifts::table
            .filter(recurrent_shifts::weekday.eq_any(vec![&today, &today.prev(), &today.next()]))
            .select(RecurringShift::as_select())
            .load(conn)?;

        let mut new_shifts = Vec::new();

        for template in templates {
            let timezone = parse_timezone(&template.timezone)?;
            let local_date = if template.weekday == today.prev() {
                date.pred_opt()
                    .ok_or_else(|| ApiError::InternalServerError("Invalid previous date"))?
            } else if template.weekday == today.next() {
                date.succ_opt()
                    .ok_or_else(|| ApiError::InternalServerError("Invalid next date"))?
            } else {
                date
            };

            let naive_dt = u32::try_from(template.start_hour)
                .ok()
                .and_then(|hour| local_date.and_hms_opt(hour, 0, 0))
                .ok_or_else(|| ApiError::InternalServerError("Invalid start hour"))?;

            let start_at: DateTime<Utc> = timezone
                .from_local_datetime(&naive_dt)
                .single()
                .ok_or_else(|| ApiError::InternalServerError("Invalid datetime in timezone"))?
                .with_timezone(&Utc);

            let end_at = start_at + chrono::Duration::hours(i64::from(template.duration));

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

                new_shifts.push(new.clone());
                diesel::insert_into(shifts::table)
                    .values(&new)
                    .execute(conn)?;
            }
        }

        Ok(new_shifts)
    }
}
