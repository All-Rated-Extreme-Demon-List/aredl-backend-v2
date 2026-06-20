use crate::aredl::bounty::Bounty;
use crate::notifications::WebsocketNotification;
use crate::{
    app_data::db::DbConnection,
    aredl::levels::LevelStatus,
    aredl::submissions::{status::SubmissionsEnabled, Submission, SubmissionStatus},
    auth::{Authenticated, Permission},
    error_handler::ApiError,
    providers::ProvidersAppState,
    schema::aredl::{levels, submissions},
};
use diesel::{
    Connection as _, ExpressionMethods as _, OptionalExtension as _, QueryDsl as _,
    RunQueryDsl as _, SelectableHelper as _,
};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Insertable, ToSchema)]
#[diesel(table_name=submissions, check_for_backend(Pg))]
pub struct SubmissionPostUser {
    /// UUID of the level this record is on.
    pub level_id: Uuid,
    /// Set to `true` if this completion is on a mobile device.
    pub mobile: bool,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Completion video URL.
    ///
    /// The provider is enforced and the URL is stored in a standardized canonical form.
    /// See [Allowed video URL types](#allowed-video-url-types).
    pub video_url: String,
    /// Raw footage URL (optional).
    ///
    /// Only requires a valid URL (the site is not enforced). If the URL matches a recognized provider
    /// it is standardized, otherwise it is stored as-is.
    /// See [Allowed video URL types](#allowed-video-url-types).
    pub raw_url: Option<String>,
    /// The mod menu used in this record
    pub mod_menu: Option<String>,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Insertable, ToSchema, Default)]
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
    /// Completion video URL.
    ///
    /// The provider is enforced and the URL is stored in a standardized canonical form.
    /// See [Allowed video URL types](#allowed-video-url-types).
    pub video_url: String,
    /// Raw footage URL (optional).
    ///
    /// Only requires a valid URL (the site is not enforced). If the URL matches a recognized provider
    /// it is standardized, otherwise it is stored as-is.
    /// See [Allowed video URL types](#allowed-video-url-types).
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
    /// Completion video URL.
    ///
    /// The provider is enforced and the URL is stored in a standardized canonical form.
    /// See [Allowed video URL types](#allowed-video-url-types).
    pub video_url: String,
    /// Raw footage URL (optional).
    ///
    /// Only requires a valid URL (the site is not enforced). If the URL matches a recognized provider
    /// it is standardized, otherwise it is stored as-is.
    /// See [Allowed video URL types](#allowed-video-url-types).
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
    /// UUID of the user reviewing the submission.
    pub reviewer_id: Option<Uuid>,
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
        }
    }
}

impl SubmissionInsert {
    pub fn from_user(
        conn: &mut DbConnection,
        body: SubmissionPostUser,
        authenticated: &Authenticated,
    ) -> Result<Self, ApiError> {
        let active_bounties = Bounty::find_active_by_level(conn, body.level_id)?;
        Ok(SubmissionInsert {
            submitted_by: authenticated.user_id,
            level_id: body.level_id,
            mobile: body.mobile,
            ldm_id: body.ldm_id,
            video_url: body.video_url,
            raw_url: body.raw_url,
            mod_menu: body.mod_menu,
            user_notes: body.user_notes,
            priority: authenticated.is_aredl_plus(conn)? || !active_bounties.is_empty(),
            status: SubmissionStatus::Pending,
            ..Default::default()
        })
    }

    pub fn from_mod(
        conn: &mut DbConnection,
        body: SubmissionPostMod,
        authenticated: &Authenticated,
    ) -> Result<Self, ApiError> {
        let submitted_by = body.submitted_by.unwrap_or(authenticated.user_id);

        if !authenticated.has_permission(conn, Permission::SubmissionReviewFull)?
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
            priority: body.priority.unwrap_or(false),
            status: body.status.unwrap_or(SubmissionStatus::Pending),
            reviewer_notes: body.reviewer_notes,
            reviewer_id: Some(authenticated.user_id),
        })
    }
}

impl Submission {
    pub fn create(
        conn: &mut DbConnection,
        mut submission_body: SubmissionPostMod,
        authenticated: &Authenticated,
        providers: &ProvidersAppState,
        notify_tx: &broadcast::Sender<WebsocketNotification>,
    ) -> Result<Self, ApiError> {
        submission_body.video_url = providers
            .validate_completion_video_url(&submission_body.video_url)
            .map_err(|mut e| {
                e.error_message = format!("Invalid completion video URL: {}", e.error_message);
                e
            })?;

        if let Some(raw_url) = submission_body.raw_url.as_ref() {
            submission_body.raw_url = Some(providers.validate_raw_footage_url(raw_url).map_err(
                |mut e| {
                    e.error_message = format!("Invalid raw footage URL: {}", e.error_message);
                    e
                },
            )?);
        }

        let submission = conn.transaction(|connection| -> Result<Self, ApiError> {
            let inserted_submission =
                SubmissionInsert::from_mod(connection, submission_body, authenticated)?;

            if authenticated.user_id == inserted_submission.submitted_by
                && !(SubmissionsEnabled::is_enabled(connection)?)
            {
                return Err(ApiError::Forbidden("Submissions are currently disabled"));
            }

            // check if any submissions exist already
            let exists_submission = submissions::table
                .filter(submissions::submitted_by.eq(inserted_submission.submitted_by))
                .filter(submissions::level_id.eq(inserted_submission.level_id))
                .select(submissions::id)
                .first::<Uuid>(connection)
                .optional()?;

            if exists_submission.is_some() {
                return Err(ApiError::Conflict(
                    "You already have a submission for this level",
                ));
            }

            // check that this level exists, accepts submissions, and
            // raw footage is provided when required
            let level_info = levels::table
                .filter(levels::id.eq(inserted_submission.level_id))
                .select((
                    levels::status,
                    levels::position,
                    levels::requires_raw_footage,
                ))
                .first::<(LevelStatus, Option<i32>, bool)>(connection)
                .optional()?;

            match level_info {
                None => return Err(ApiError::NotFound("Could not find this level")),
                Some((status, position, requires_raw_footage)) => {
                    if status == LevelStatus::Legacy {
                        return Err(ApiError::UnprocessableEntity(
                            "This level is on the legacy list and is not accepting records.",
                        ));
                    }
                    if status == LevelStatus::Removed {
                        return Err(ApiError::Gone("This level has been removed from the list."));
                    }

                    let raw_is_required = match status {
                        LevelStatus::Pending => requires_raw_footage,
                        LevelStatus::MainList => position.is_some_and(|position| position <= 400),
                        LevelStatus::Legacy | LevelStatus::Removed => false,
                    };

                    if raw_is_required && inserted_submission.raw_url.is_none() {
                        return Err(ApiError::UnprocessableEntity(
                            "This level requires raw footage",
                        ));
                    }
                }
            }

            let submission = diesel::insert_into(submissions::table)
                .values(&inserted_submission)
                .returning(Self::as_select())
                .get_result(connection)?;

            Ok(submission)
        })?;

        WebsocketNotification::send(notify_tx, "SUBMISSION_CREATED", &submission);

        Ok(submission)
    }
}
