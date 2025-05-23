use crate::{
    aredl::{
        records::Record,
        shifts::ShiftStatus,
        submissions::{history::SubmissionHistory, *},
    },
    auth::Authenticated,
    db::DbAppState,
    error_handler::ApiError,
    schema::aredl::{
        levels, records, shifts, submission_history, submissions, submissions_with_priority,
    },
    users::me::notifications::{Notification, NotificationType},
};
use actix_web::web;
use chrono::Utc;
use diesel::{
    Connection, ExpressionMethods, OptionalExtension, PgConnection, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema, Debug)]
pub struct ReviewerNotes {
    pub notes: Option<String>,
}

impl Submission {
    pub fn increment_user_shift(conn: &mut PgConnection, user_id: Uuid) -> Result<(), ApiError> {
        let now = Utc::now();

        let running_shift_id = shifts::table
            .filter(shifts::user_id.eq(user_id))
            .filter(shifts::status.eq(ShiftStatus::Running))
            .filter(shifts::start_at.le(now))
            .filter(shifts::end_at.gt(now))
            .order(shifts::start_at.asc())
            .select(shifts::id)
            .first::<Uuid>(conn)
            .optional()?;

        if let Some(shift_id) = running_shift_id {
            let (completed_count, target_count) =
                diesel::update(shifts::table.filter(shifts::id.eq(shift_id)))
                    .set((
                        shifts::completed_count.eq(shifts::completed_count + 1),
                        shifts::updated_at.eq(now),
                    ))
                    .returning((shifts::completed_count, shifts::target_count))
                    .get_result::<(i32, i32)>(conn)?;

            if completed_count >= target_count {
                diesel::update(shifts::table.filter(shifts::id.eq(shift_id)))
                    .set(shifts::status.eq(ShiftStatus::Completed))
                    .execute(conn)?;
            }
        }
        Ok(())
    }
    pub fn accept(
        db: web::Data<Arc<DbAppState>>,
        id: Uuid,
        reviewer_id: Uuid,
        notes: Option<String>,
    ) -> Result<Record, ApiError> {
        let conn = &mut db.connection()?;
        conn.transaction(|connection| -> Result<Record, ApiError> {
            let updated = submissions::table
                .filter(submissions::id.eq(id))
                .select(Submission::as_select())
                .first::<Submission>(connection)?;

            let existing_record_id = records::table
                .filter(records::submitted_by.eq(updated.submitted_by))
                .filter(records::level_id.eq(updated.level_id))
                .select(records::id)
                .first::<Uuid>(connection)
                .optional()?;

            let record_data = (
                records::mobile.eq(updated.mobile),
                records::ldm_id.eq(updated.ldm_id),
                records::video_url.eq(updated.video_url),
                records::raw_url.eq(updated.raw_url),
                records::reviewer_id.eq(Some(reviewer_id)),
                records::mod_menu.eq(updated.mod_menu),
                records::user_notes.eq(updated.user_notes),
                records::reviewer_notes.eq(notes.clone()),
                records::updated_at.eq(chrono::Utc::now()),
            );

            let inserted = if let Some(record_id) = existing_record_id {
                diesel::update(records::table.filter(records::id.eq(record_id)))
                    .set(record_data)
                    .returning(Record::as_select())
                    .get_result::<Record>(connection)?
            } else {
                diesel::insert_into(records::table)
                    .values((
                        records::submitted_by.eq(updated.submitted_by),
                        records::level_id.eq(updated.level_id),
                        records::created_at.eq(chrono::Utc::now()),
                        record_data,
                    ))
                    .returning(Record::as_select())
                    .get_result::<Record>(connection)?
            };

            // Log submission history
            let history = SubmissionHistory {
                id: Uuid::new_v4(),
                submission_id: updated.id,
                record_id: Some(inserted.id),
                status: SubmissionStatus::Accepted,
                reviewer_notes: notes,
                reviewer_id: Some(reviewer_id),
                user_notes: None,
                timestamp: chrono::Utc::now(),
            };
            diesel::insert_into(submission_history::table)
                .values(&history)
                .execute(connection)?;

            let level_name = levels::table
                .filter(levels::id.eq(updated.level_id))
                .select(levels::name)
                .first::<String>(connection)?;

            Self::increment_user_shift(connection, reviewer_id)?;

            let content = format!("Your submissions for {:?} has been accepted!", level_name);
            Notification::create(
                connection,
                inserted.submitted_by,
                content,
                NotificationType::Success,
            )?;

            diesel::delete(submissions::table)
                .filter(submissions::id.eq(id))
                .execute(connection)?;

            Ok(inserted)
        })
    }

    pub fn reject(
        db: web::Data<Arc<DbAppState>>,
        id: Uuid,
        authenticated: Authenticated,
        notes: Option<String>,
    ) -> Result<SubmissionResolved, ApiError> {
        let connection = &mut db.connection()?;

        let update_timestamp = chrono::Utc::now();

        let user_id = authenticated.user_id;

        let new_data = (
            submissions::status.eq(SubmissionStatus::Denied),
            submissions::reviewer_id.eq(authenticated.user_id),
            submissions::reviewer_notes.eq(notes.clone()),
            submissions::updated_at.eq(update_timestamp.clone()),
        );

        let current_status = submissions::table
            .filter(submissions::id.eq(id))
            .select(submissions::status)
            .first::<SubmissionStatus>(connection)?;

        if current_status == SubmissionStatus::Denied {
            return Err(ApiError::new(
                409,
                "This submission is already in the denied state!",
            ));
        }

        let updated_submission = diesel::update(submissions::table)
            .filter(submissions::id.eq(id))
            .set(new_data)
            .returning(Submission::as_select())
            .get_result::<Submission>(connection)?;

        let resolved_updated_submission =
            SubmissionResolved::resolve_from_id(updated_submission.id, db, authenticated)?;

        // Log submission history
        let history = SubmissionHistory {
            id: Uuid::new_v4(),
            submission_id: resolved_updated_submission.id,
            record_id: None,
            status: SubmissionStatus::Denied,
            reviewer_notes: notes,
            reviewer_id: Some(user_id),
            user_notes: None,
            timestamp: update_timestamp,
        };
        diesel::insert_into(submission_history::table)
            .values(&history)
            .execute(connection)?;

        Self::increment_user_shift(connection, user_id)?;

        let content: String = format!(
            "Your submission for {:?} has been denied.",
            resolved_updated_submission.level.name
        );
        Notification::create(
            connection,
            resolved_updated_submission.submitted_by.id,
            content,
            NotificationType::Failure,
        )?;
        Ok(resolved_updated_submission)
    }

    pub fn under_consideration(
        db: web::Data<Arc<DbAppState>>,
        id: Uuid,
        authenticated: Authenticated,
        notes: Option<String>,
    ) -> Result<SubmissionResolved, ApiError> {
        let connection = &mut db.connection()?;

        let update_timestamp = chrono::Utc::now();

        let user_id = authenticated.user_id;

        let new_data = (
            submissions::status.eq(SubmissionStatus::UnderConsideration),
            submissions::reviewer_id.eq(authenticated.user_id),
            submissions::reviewer_notes.eq(&notes),
            submissions::updated_at.eq(update_timestamp.clone()),
        );

        let current_status = submissions::table
            .filter(submissions::id.eq(id))
            .select(submissions::status)
            .first::<SubmissionStatus>(connection)?;

        if current_status == SubmissionStatus::UnderConsideration {
            return Err(ApiError::new(
                409,
                "This submission is already in the under consideration state!",
            ));
        }

        let updated_submission = diesel::update(submissions::table)
            .filter(submissions::id.eq(id))
            .set(new_data)
            .returning(Submission::as_select())
            .get_result::<Submission>(connection)?;

        let resolved_updated_submission =
            SubmissionResolved::resolve_from_id(updated_submission.id, db, authenticated)?;

        // Log submission history
        let history = SubmissionHistory {
            id: Uuid::new_v4(),
            submission_id: resolved_updated_submission.id,
            record_id: None,
            status: SubmissionStatus::UnderConsideration,
            reviewer_notes: notes,
            reviewer_id: Some(user_id),
            user_notes: None,
            timestamp: update_timestamp,
        };
        diesel::insert_into(submission_history::table)
            .values(&history)
            .execute(connection)?;

        Self::increment_user_shift(connection, user_id)?;

        let content = format!(
            "Your submission for {:?} has been placed under consideration.",
            resolved_updated_submission.level.name
        );
        Notification::create(
            connection,
            resolved_updated_submission.submitted_by.id,
            content,
            NotificationType::Info,
        )?;
        Ok(resolved_updated_submission)
    }

    pub fn unclaim(
        db: web::Data<Arc<DbAppState>>,
        id: Uuid,
        authenticated: Authenticated,
    ) -> Result<SubmissionResolved, ApiError> {
        let connection = &mut db.connection()?;

        let new_data = (
            submissions::status.eq(SubmissionStatus::Pending),
            submissions::reviewer_id.eq::<Option<Uuid>>(None),
            submissions::updated_at.eq(chrono::Utc::now()),
        );

        let current_status = submissions::table
            .filter(submissions::id.eq(id))
            .select(submissions::status)
            .first::<SubmissionStatus>(connection)?;

        if current_status == SubmissionStatus::Pending {
            return Err(ApiError::new(409, "This submission is not claimed!"));
        }

        let updated_submission = diesel::update(submissions::table)
            .filter(submissions::id.eq(id))
            .set(new_data)
            .returning(Submission::as_select())
            .get_result::<Submission>(connection)?;

        let resolved_updated_submission =
            SubmissionResolved::resolve_from_id(updated_submission.id, db, authenticated)?;

        Ok(resolved_updated_submission)
    }
}

impl SubmissionResolved {
    pub fn claim_highest_priority(
        db: web::Data<Arc<DbAppState>>,
        authenticated: Authenticated,
    ) -> Result<SubmissionResolved, ApiError> {
        db.connection()?
            .transaction(|conn| -> Result<SubmissionResolved, ApiError> {
                let next_id: Uuid = submissions_with_priority::table
                    .filter(submissions_with_priority::status.eq(SubmissionStatus::Pending))
                    .for_update()
                    .skip_locked()
                    .order((
                        submissions_with_priority::priority_value.desc(),
                        submissions_with_priority::created_at.asc(),
                    ))
                    .select(submissions_with_priority::id)
                    .first(conn)?;

                diesel::update(submissions::table.filter(submissions::id.eq(next_id)))
                    .set((
                        submissions::status.eq(SubmissionStatus::Claimed),
                        submissions::reviewer_id.eq(authenticated.user_id),
                        submissions::updated_at.eq(chrono::Utc::now()),
                    ))
                    .execute(conn)?;

                let resolved = SubmissionResolved::resolve_from_id(next_id, db, authenticated)?;

                Ok(resolved)
            })
    }
}
