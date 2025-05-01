use crate::{
    aredl::{
        records::Record,
        submissions::{history::SubmissionHistory, *},
    },
    auth::Authenticated,
    custom_schema::aredl_submissions_with_priority,
    db::DbAppState,
    error_handler::ApiError,
    schema::{aredl_levels, aredl_records, aredl_submissions, submission_history},
    users::me::notifications::{Notification, NotificationType},
};
use actix_web::web;
use diesel::{
    Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, ToSchema)]
pub struct ReviewerNotes {
    pub notes: Option<String>,
}

impl Submission {
    pub fn accept(
        db: web::Data<Arc<DbAppState>>,
        id: Uuid,
        reviewer_id: Uuid,
        notes: Option<String>,
    ) -> Result<Record, ApiError> {
        let conn = &mut db.connection()?;
        conn.transaction(|connection| -> Result<Record, ApiError> {
            let updated = aredl_submissions::table
                .filter(aredl_submissions::id.eq(id))
                .select(Submission::as_select())
                .first::<Submission>(connection)?;

            let existing_record_id = aredl_records::table
                .filter(aredl_records::submitted_by.eq(updated.submitted_by))
                .filter(aredl_records::level_id.eq(updated.level_id))
                .select(aredl_records::id)
                .first::<Uuid>(connection)
                .optional()?;

            let record_data = (
                aredl_records::mobile.eq(updated.mobile),
                aredl_records::ldm_id.eq(updated.ldm_id),
                aredl_records::video_url.eq(updated.video_url),
                aredl_records::raw_url.eq(updated.raw_url),
                aredl_records::reviewer_id.eq(Some(reviewer_id)),
                aredl_records::mod_menu.eq(updated.mod_menu),
                aredl_records::user_notes.eq(updated.user_notes),
                aredl_records::reviewer_notes.eq(notes.clone()),
                aredl_records::updated_at.eq(chrono::Utc::now().naive_utc()),
            );

            let inserted = if let Some(record_id) = existing_record_id {
                diesel::update(aredl_records::table.filter(aredl_records::id.eq(record_id)))
                    .set(record_data)
                    .returning(Record::as_select())
                    .get_result::<Record>(connection)?
            } else {
                diesel::insert_into(aredl_records::table)
                    .values((
                        aredl_records::submitted_by.eq(updated.submitted_by),
                        aredl_records::level_id.eq(updated.level_id),
                        aredl_records::created_at.eq(chrono::Utc::now().naive_utc()),
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
                status: SubmissionStatus::Claimed,
                reviewer_notes: notes,
                reviewer_id: Some(reviewer_id),
                user_notes: None,
                timestamp: chrono::Utc::now().naive_utc(),
            };
            diesel::insert_into(submission_history::table)
                .values(&history)
                .execute(connection)?;

            let level_name = aredl_levels::table
                .filter(aredl_levels::id.eq(updated.level_id))
                .select(aredl_levels::name)
                .first::<String>(connection)?;

            let content = format!("Your submissions for {:?} has been accepted!", level_name);
            Notification::create(
                connection,
                inserted.submitted_by,
                content,
                NotificationType::Success,
            )?;

            diesel::delete(aredl_submissions::table)
                .filter(aredl_submissions::id.eq(id))
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

        let update_timestamp = chrono::Utc::now().naive_utc();

        let user_id = authenticated.user_id;

        let new_data = (
            aredl_submissions::status.eq(SubmissionStatus::Denied),
            aredl_submissions::reviewer_id.eq(authenticated.user_id),
            aredl_submissions::reviewer_notes.eq(notes.clone()),
            aredl_submissions::updated_at.eq(update_timestamp.clone()),
        );

        let updated_submission = diesel::update(aredl_submissions::table)
            .filter(aredl_submissions::id.eq(id))
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

        let content = format!(
            "Your record on {:?} has been denied...",
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

        let update_timestamp = chrono::Utc::now().naive_utc();

        let user_id = authenticated.user_id;

        let new_data = (
            aredl_submissions::status.eq(SubmissionStatus::UnderConsideration),
            aredl_submissions::reviewer_id.eq(authenticated.user_id),
            aredl_submissions::reviewer_notes.eq(notes.clone()),
            aredl_submissions::updated_at.eq(update_timestamp.clone()),
        );

        let updated_submission = diesel::update(aredl_submissions::table)
            .filter(aredl_submissions::id.eq(id))
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

        let content = format!(
            "Your record on {:?} has been placed under consideration.",
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
            aredl_submissions::status.eq(SubmissionStatus::Pending),
            aredl_submissions::reviewer_id.eq::<Option<Uuid>>(None),
            aredl_submissions::updated_at.eq(chrono::Utc::now().naive_utc()),
        );

        let updated_submission = diesel::update(aredl_submissions::table)
            .filter(aredl_submissions::id.eq(id))
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
                let next_id: Uuid = aredl_submissions_with_priority::table
                    .filter(aredl_submissions_with_priority::status.eq(SubmissionStatus::Pending))
                    .for_update()
                    .skip_locked()
                    .order((
                        aredl_submissions_with_priority::priority_value.desc(),
                        aredl_submissions_with_priority::created_at.asc(),
                    ))
                    .select(aredl_submissions_with_priority::id)
                    .first(conn)?;

                diesel::update(aredl_submissions::table.filter(aredl_submissions::id.eq(next_id)))
                    .set((
                        aredl_submissions::status.eq(SubmissionStatus::Claimed),
                        aredl_submissions::reviewer_id.eq(authenticated.user_id),
                        aredl_submissions::updated_at.eq(chrono::Utc::now().naive_utc()),
                    ))
                    .execute(conn)?;

                let resolved = SubmissionResolved::resolve_from_id(next_id, db, authenticated)?;

                Ok(resolved)
            })
    }
}
