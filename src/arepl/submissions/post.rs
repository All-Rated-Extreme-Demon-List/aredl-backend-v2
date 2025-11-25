use crate::{
    app_data::db::DbConnection,
    arepl::submissions::{status::SubmissionsEnabled, *},
    auth::{Authenticated, Permission},
    error_handler::ApiError,
    schema::arepl::{levels, submissions},
};
use diesel::{
    Connection, ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper,
};
use is_url::is_url;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Insertable, ToSchema, Default)]
#[diesel(table_name=submissions, check_for_backend(Pg))]
pub struct SubmissionInsert {
    /// UUID of the user submitting the record.
    pub submitted_by: Uuid,
    /// UUID of the level this record is on.
    pub level_id: Uuid,
    /// Set to `true` if this completion is on a mobile device.
    pub mobile: bool,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Completion time of the record in milliseconds.
    pub completion_time: i64,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// The mod menu used in this record
    pub mod_menu: Option<String>,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
    /// Whether this submission has priority in the review queue.
    pub priority: bool,
    /// Status of the submission
    pub status: SubmissionStatus,
    /// Reviewer notes for the submission.
    pub reviewer_notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Insertable, ToSchema)]
#[diesel(table_name=submissions, check_for_backend(Pg))]

pub struct SubmissionPostMod {
    /// [MOD ONLY] UUID of the user submitting the record.
    pub submitted_by: Option<Uuid>,
    /// UUID of the level this record is on.
    pub level_id: Uuid,
    /// Set to `true` if this completion is on a mobile device.
    pub mobile: bool,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Completion time of the record in milliseconds.
    pub completion_time: i64,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// The mod menu used in this record
    pub mod_menu: Option<String>,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
    /// [MOD ONLY] Whether this submission has priority in the review queue.
    pub priority: Option<bool>,
    /// [MOD ONLY] Initial status of the submission
    pub status: Option<SubmissionStatus>,
    /// [MOD ONLY] Reviewer notes for the submission.
    pub reviewer_notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Insertable, ToSchema)]
#[diesel(table_name=submissions, check_for_backend(Pg))]
pub struct SubmissionPostUser {
    /// UUID of the level this record is on.
    pub level_id: Uuid,
    /// Set to `true` if this completion is on a mobile device.
    pub mobile: bool,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Video link of the completion.
    pub video_url: String,
    /// Completion time of the record in milliseconds.
    pub completion_time: i64,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// The mod menu used in this record
    pub mod_menu: Option<String>,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
}

impl SubmissionPostMod {
    pub fn downgrade(self) -> SubmissionPostUser {
        SubmissionPostUser {
            level_id: self.level_id,
            mobile: self.mobile,
            ldm_id: self.ldm_id,
            video_url: self.video_url,
            raw_url: self.raw_url,
            mod_menu: self.mod_menu,
            user_notes: self.user_notes,
            completion_time: self.completion_time,
        }
    }
}

impl SubmissionInsert {
    pub fn from_user(
        conn: &mut DbConnection,
        body: SubmissionPostUser,
        authenticated: &Authenticated,
    ) -> Result<Self, ApiError> {
        Ok(SubmissionInsert {
            submitted_by: authenticated.user_id,
            level_id: body.level_id,
            mobile: body.mobile,
            ldm_id: body.ldm_id,
            video_url: body.video_url,
            raw_url: body.raw_url,
            mod_menu: body.mod_menu,
            user_notes: body.user_notes,
            completion_time: body.completion_time,
            priority: authenticated.is_aredl_plus(conn)?,
            status: SubmissionStatus::Pending,
            reviewer_notes: None,
        })
    }

    pub fn from_mod(
        conn: &mut DbConnection,
        body: SubmissionPostMod,
        authenticated: &Authenticated,
    ) -> Result<Self, ApiError> {
        let submitted_by = body.submitted_by.unwrap_or(authenticated.user_id);

        if !authenticated.has_permission(conn, Permission::SubmissionReview)?
            || submitted_by == authenticated.user_id
        {
            return SubmissionInsert::from_user(conn, body.downgrade(), authenticated);
        }

        Ok(SubmissionInsert {
            submitted_by,
            level_id: body.level_id,
            mobile: body.mobile,
            ldm_id: body.ldm_id,
            video_url: body.video_url,
            raw_url: body.raw_url,
            mod_menu: body.mod_menu,
            user_notes: body.user_notes,
            completion_time: body.completion_time,
            priority: authenticated.is_aredl_plus(conn)?,
            status: SubmissionStatus::Pending,
            reviewer_notes: body.reviewer_notes,
        })
    }
}

impl Submission {
    pub fn create(
        conn: &mut DbConnection,
        submission_body: SubmissionPostMod,
        authenticated: Authenticated,
    ) -> Result<Self, ApiError> {
        if !is_url(&submission_body.video_url) {
            return Err(ApiError::new(
                400,
                "Completion video link is not a valid URL!",
            ));
        }

        if let Some(raw_url) = submission_body.raw_url.as_ref() {
            if !is_url(raw_url) {
                return Err(ApiError::new(400, "Raw footage link is not a valid URL!"));
            }
        }

        conn.transaction(|connection| -> Result<Self, ApiError> {
            let inserted_submission =
                SubmissionInsert::from_mod(connection, submission_body, &authenticated)?;

            if authenticated.user_id == inserted_submission.submitted_by
                && !(SubmissionsEnabled::is_enabled(connection)?)
            {
                return Err(ApiError::new(400, "Submissions are currently disabled"));
            }

            // check if any submissions exist already
            let exists_submission = submissions::table
                .filter(submissions::submitted_by.eq(authenticated.user_id))
                .filter(submissions::level_id.eq(inserted_submission.level_id))
                .select(submissions::id)
                .first::<Uuid>(connection)
                .optional()?;

            if exists_submission.is_some() {
                return Err(ApiError::new(
                    409,
                    "You already have a submission for this level",
                ));
            }

            // check that this level exists, is not legacy, and
            // raw footage is provided for ranks 400+
            let level_info = levels::table
                .filter(levels::id.eq(inserted_submission.level_id))
                .select((levels::legacy, levels::position))
                .first::<(bool, i32)>(connection)
                .optional()?;

            match level_info {
                None => return Err(ApiError::new(404, "Could not find this level")),
                Some((legacy, pos)) => {
                    if legacy == true {
                        return Err(ApiError::new(
                            400,
                            "This level is on the legacy list and is not accepting records.",
                        ));
                    }
                    if pos <= 400 && inserted_submission.raw_url.is_none() {
                        return Err(ApiError::new(
                            400,
                            "This level is top 400 and requires raw footage",
                        ));
                    }
                }
            }

            let submission = diesel::insert_into(submissions::table)
                .values(&inserted_submission)
                .returning(Self::as_select())
                .get_result(connection)?;

            Ok(submission)
        })
    }
}
