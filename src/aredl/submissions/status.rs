use crate::{
    aredl::records::Record,
    aredl::submissions::*,
    auth::Authenticated,
    db::DbAppState,
    error_handler::ApiError,
    schema::{
        aredl_levels, aredl_records, aredl_submissions, submission_history,
    },
    users::
        me::notifications::{Notification, NotificationType},
    custom_schema::aredl_submissions_with_priority
};
use actix_web::web;
use diesel::{
    Connection, ExpressionMethods, OptionalExtension,
    QueryDsl, RunQueryDsl, SelectableHelper,
};
use std::sync::Arc;
use uuid::Uuid;

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
                aredl_records::reviewer_notes.eq(notes),
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
                rejection_reason: None,
                timestamp: chrono::Utc::now().naive_utc(),
            };
            diesel::insert_into(submission_history::table)
                .values(&history)
                .execute(connection)?;

            let level_name = aredl_levels::table
                .filter(aredl_levels::id.eq(updated.level_id))
                .select(aredl_levels::name)
                .first::<String>(connection)?;

            let content = format!("Your record on {:?} has been accepted!", level_name);
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

        let new_data = (
            aredl_submissions::status.eq(SubmissionStatus::Denied),
            aredl_submissions::reviewer_id.eq(authenticated.user_id),
            aredl_submissions::reviewer_notes.eq(notes.clone()),
        );

        let new_record = diesel::update(aredl_submissions::table)
            .filter(aredl_submissions::id.eq(id))
            .set(new_data)
            .returning(Submission::as_select())
            .get_result::<Submission>(connection)?;

        let upgraded = SubmissionResolved::from(new_record.clone(), db, None)?;

        // Log submission history
        let history = SubmissionHistory {
            id: Uuid::new_v4(),
            submission_id: new_record.id,
            record_id: None,
            status: SubmissionStatus::Denied,
            rejection_reason: notes,
            timestamp: chrono::Utc::now().naive_utc(),
        };
        diesel::insert_into(submission_history::table)
            .values(&history)
            .execute(connection)?;

        let content = format!(
            "Your record on {:?} has been denied...",
            upgraded.level.name
        );
        Notification::create(
            connection,
            upgraded.submitted_by.id,
            content,
            NotificationType::Failure,
        )?;
        Ok(upgraded)
    }

    pub fn under_consideration(
        db: web::Data<Arc<DbAppState>>,
        id: Uuid,
        authenticated: Authenticated,
        notes: Option<String>,
    ) -> Result<SubmissionResolved, ApiError> {
        let connection = &mut db.connection()?;

        let new_data = (
            aredl_submissions::status.eq(SubmissionStatus::UnderConsideration),
            aredl_submissions::reviewer_id.eq(authenticated.user_id),
            aredl_submissions::reviewer_notes.eq(notes.clone()),
        );

        let new_record = diesel::update(aredl_submissions::table)
            .filter(aredl_submissions::id.eq(id))
            .set(new_data)
            .returning(Submission::as_select())
            .get_result::<Submission>(connection)?;

        let upgraded = SubmissionResolved::from(new_record.clone(), db, None)?;

        // Log submission history
        let history = SubmissionHistory {
            id: Uuid::new_v4(),
            submission_id: new_record.id,
            record_id: None,
            status: SubmissionStatus::UnderConsideration,
            rejection_reason: None,
            timestamp: chrono::Utc::now().naive_utc(),
        };
        diesel::insert_into(submission_history::table)
            .values(&history)
            .execute(connection)?;

        let content = format!(
            "Your record on {:?} has been placed under consideration.",
            upgraded.level.name
        );
        Notification::create(
            connection,
            upgraded.submitted_by.id,
            content,
            NotificationType::Info,
        )?;
        Ok(upgraded)
    }

    pub fn unclaim(
        db: web::Data<Arc<DbAppState>>,
        id: Uuid,
    ) -> Result<SubmissionResolved, ApiError> {
        let connection = &mut db.connection()?;

        let new_data = (
            aredl_submissions::status.eq(SubmissionStatus::Pending),
            aredl_submissions::reviewer_id.eq::<Option<Uuid>>(None),
        );

        let new_record = diesel::update(aredl_submissions::table)
            .filter(aredl_submissions::id.eq(id))
            .set(new_data)
            .returning(Submission::as_select())
            .get_result::<Submission>(connection)?;

        let upgraded = SubmissionResolved::from(new_record.clone(), db, None)?;

        // Log submission history
        let history = SubmissionHistory {
            id: Uuid::new_v4(),
            submission_id: new_record.id,
            record_id: None,
            status: SubmissionStatus::Pending,
            rejection_reason: None,
            timestamp: chrono::Utc::now().naive_utc(),
        };
        diesel::insert_into(submission_history::table)
            .values(&history)
            .execute(connection)?;

        Ok(upgraded)
    }
}

impl SubmissionResolved {
    pub fn find_highest_priority(
        db: web::Data<Arc<DbAppState>>,
        user: Uuid,
    ) -> Result<SubmissionResolved, ApiError> {
        let conn = &mut db.connection()?;
        let new_data = (
            aredl_submissions::status.eq(SubmissionStatus::Claimed),
            aredl_submissions::reviewer_id.eq(user),
        );

        // TODO: maybe this could become one super clean query?
        let highest_priority_id = aredl_submissions_with_priority::table
            .filter(aredl_submissions_with_priority::status.eq(SubmissionStatus::Pending))
            .select((
                aredl_submissions_with_priority::id,
                aredl_submissions_with_priority::priority_value,
            ))
            .order(aredl_submissions_with_priority::priority_value.desc())
            .limit(1)
            .first::<(Uuid, i64)>(conn)?;

        // we don't really need to return the priority value here
        let submission = diesel::update(
            aredl_submissions::table.filter(aredl_submissions::id.eq(highest_priority_id.0)),
        )
        .set(new_data)
        .returning(Submission::as_select())
        .get_result(conn)?;

        let upgraded = SubmissionResolved::from(submission, db, Some(highest_priority_id.1))?;

        Ok(upgraded)
    }
}
