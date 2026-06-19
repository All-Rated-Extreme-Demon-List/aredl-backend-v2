use crate::{
    app_data::db::DbConnection,
    aredl::levels::LevelStatus,
    aredl::submissions::{status::SubmissionsEnabled, *},
    auth::{Authenticated, Permission},
    error_handler::ApiError,
    notifications::WebsocketNotification,
    providers::ProvidersAppState,
    schema::{
        aredl::{levels, submissions},
        shifts, users,
    },
    shifts::{Shift, ShiftStatus},
    users::me::notifications::{Notification, NotificationType},
};
use chrono::Utc;
use diesel::Connection;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use tokio::sync::broadcast;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, AsChangeset, Default, ToSchema, Clone, PartialEq)]
#[diesel(table_name=submissions, check_for_backend(Pg))]
pub struct SubmissionPatchUser {
    /// Whether the record was completed on mobile or not.
    pub mobile: Option<bool>,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Completion video URL.
    ///
    /// The provider is enforced and the URL is stored in a standardized canonical form.
    /// See [Allowed video URL types](#allowed-video-url-types).
    pub video_url: Option<String>,
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

#[derive(Serialize, Deserialize, Debug, AsChangeset, Default, ToSchema, Clone, PartialEq)]
#[diesel(table_name=submissions, check_for_backend(Pg))]
pub struct SubmissionPatchMod {
    /// Whether the record was completed on mobile or not.
    pub mobile: Option<bool>,
    /// ID of the LDM used for the record, if any.
    pub ldm_id: Option<i32>,
    /// Completion video URL.
    ///
    /// The provider is enforced and the URL is stored in a standardized canonical form.
    /// See [Allowed video URL types](#allowed-video-url-types).
    pub video_url: Option<String>,
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
    /// [MOD ONLY] The status of the submission
    pub status: Option<SubmissionStatus>,
    /// [MOD ONLY] Whether the record was submitted as a priority record.
    pub priority: Option<bool>,
    /// [MOD ONLY] Notes given by the reviewer when reviewing the record.
    pub reviewer_notes: Option<String>,
    /// [MOD ONLY] Private notes given by the reviewer when reviewing the record.
    pub private_reviewer_notes: Option<String>,
    /// [MOD ONLY] Whether or not this submission should be locked
    pub locked: Option<bool>,
}

impl Submission {
    pub fn update_user_shift(
        conn: &mut DbConnection,
        user_id: Uuid,
        old_status: SubmissionStatus,
        new_status: SubmissionStatus,
    ) -> Result<Option<Shift>, ApiError> {
        let from_ok = matches!(
            old_status,
            SubmissionStatus::Pending
                | SubmissionStatus::Claimed
                | SubmissionStatus::UnderConsideration
                | SubmissionStatus::UnderReview
        );
        let to_ok = matches!(
            new_status,
            SubmissionStatus::Accepted
                | SubmissionStatus::Denied
                | SubmissionStatus::UnderConsideration
        );
        if from_ok && to_ok && old_status != new_status {
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
                let updated_shift = diesel::update(shifts::table.filter(shifts::id.eq(shift_id)))
                    .set((
                        shifts::completed_count.eq(shifts::completed_count + 1),
                        shifts::updated_at.eq(now),
                    ))
                    .returning(Shift::as_select())
                    .get_result::<Shift>(conn)?;

                if updated_shift.completed_count >= updated_shift.target_count {
                    diesel::update(shifts::table.filter(shifts::id.eq(shift_id)))
                        .set(shifts::status.eq(ShiftStatus::Completed))
                        .execute(conn)?;
                    return Ok(Some(updated_shift));
                }
            }
        }
        Ok(None)
    }
}

impl SubmissionPatchUser {
    pub fn patch(
        mut patch: Self,
        id: Uuid,
        conn: &mut DbConnection,
        authenticated: Authenticated,
        providers: &ProvidersAppState,
    ) -> Result<Submission, ApiError> {
        let user = authenticated.user_id;

        if patch == Self::default() {
            return Err(ApiError::BadRequest("No changes were provided!"));
        }

        if let Some(video_url) = patch.video_url.as_ref() {
            patch.video_url = Some(providers.validate_completion_video_url(video_url).map_err(
                |mut e| {
                    e.error_message = format!("Invalid completion video URL: {}", e.error_message);
                    e
                },
            )?);
        }

        if let Some(raw_url) = patch.raw_url.as_ref() {
            patch.raw_url = Some(providers.validate_raw_footage_url(raw_url).map_err(
                |mut e| {
                    e.error_message = format!("Invalid raw footage URL: {}", e.error_message);
                    e
                },
            )?);
        }

        let submitter_ban = users::table
            .filter(users::id.eq(user))
            .select(users::ban_level)
            .first::<i32>(conn)?;

        if submitter_ban >= 2 {
            return Err(ApiError::Forbidden(
                "You have been banned from submitting records.",
            ));
        }

        let old_submission: Submission = submissions::table
            .filter(submissions::id.eq(id))
            .select(Submission::as_select())
            .first::<Submission>(conn)?;

        if old_submission.submitted_by != user {
            return Err(ApiError::Forbidden(
                "You can only edit your own submissions.",
            ));
        }

        if old_submission.locked {
            return Err(ApiError::Forbidden(
                "This submission has been locked and cannot be edited",
            ));
        }

        match old_submission.status {
            SubmissionStatus::Claimed
            | SubmissionStatus::UnderConsideration
            | SubmissionStatus::UnderReview => {
                return Err(ApiError::Conflict(
                    "This submission is currently being reviewed and cannot be edited.",
                ));
            }
            _ => {}
        }

        if !SubmissionsEnabled::is_enabled(conn)?
            && old_submission.status != SubmissionStatus::Pending
        {
            return Err(ApiError::BadRequest(
                "Submissions are currently closed. You can only edit pending submissions.",
            ));
        }

        let level_status = levels::table
            .filter(levels::id.eq(old_submission.level_id))
            .select(levels::status)
            .first::<LevelStatus>(conn)?;

        match level_status {
            LevelStatus::Legacy => {
                return Err(ApiError::UnprocessableEntity(
                    "This level is on the legacy list and is not accepting records!",
                ));
            }
            LevelStatus::Removed => {
                return Err(ApiError::Gone("This level has been removed from the list."));
            }
            _ => {}
        }

        let result = diesel::update(submissions::table)
            .filter(submissions::id.eq(id))
            .filter(submissions::submitted_by.eq(user))
            .set((
                patch.clone(),
                submissions::status.eq(SubmissionStatus::Pending),
                submissions::reviewer_id.eq::<Option<Uuid>>(None),
                submissions::reviewer_notes.eq::<Option<String>>(None),
            ))
            .returning(Submission::as_select())
            .get_result::<Submission>(conn)?;

        Ok(result)
    }
}

impl SubmissionPatchMod {
    pub fn patch(
        mut patch: Self,
        id: Uuid,
        conn: &mut DbConnection,
        authenticated: Authenticated,
        notify_tx: broadcast::Sender<WebsocketNotification>,
        providers: &ProvidersAppState,
    ) -> Result<Submission, ApiError> {
        if patch == Self::default() {
            return Err(ApiError::BadRequest("No changes were provided!"));
        }

        if let Some(video_url) = patch.video_url.as_ref() {
            // for reviewers, only validate that the URL is valid like for raw footage, provider isn't enforced
            patch.video_url = Some(providers.validate_raw_footage_url(video_url).map_err(
                |mut e| {
                    e.error_message = format!("Invalid completion video URL: {}", e.error_message);
                    e
                },
            )?);
        }

        if let Some(raw_url) = patch.raw_url.as_ref() {
            patch.raw_url = Some(providers.validate_raw_footage_url(raw_url).map_err(
                |mut e| {
                    e.error_message = format!("Invalid raw footage URL: {}", e.error_message);
                    e
                },
            )?);
        }

        let old_submission: Submission = submissions::table
            .filter(submissions::id.eq(id))
            .select(Submission::as_select())
            .first::<Submission>(conn)?;

        if old_submission.submitted_by == authenticated.user_id {
            return SubmissionPatchUser::patch(
                SubmissionPatchMod::downgrade(patch),
                id,
                conn,
                authenticated,
                providers,
            );
        }

        let is_full_staff =
            authenticated.has_permission(conn, Permission::EditNonClaimedSubmissions)?;

        if !is_full_staff
            && (old_submission.raw_url.is_some()
                || old_submission.status != SubmissionStatus::Claimed
                || old_submission.reviewer_id != Some(authenticated.user_id))
        {
            return Err(ApiError::Forbidden(
                "You do not have permission to edit this submission.",
            ));
        }

        let (result, websocket_type, completed_shift) = conn.transaction(
            |connection| -> Result<(Submission, Option<&'static str>, Option<Shift>), ApiError> {
                let updated = diesel::update(submissions::table)
                    .filter(submissions::id.eq(id))
                    .set((
                        patch.clone(),
                        submissions::reviewer_id.eq(Some(authenticated.user_id)),
                    ))
                    .returning(Submission::as_select())
                    .get_result::<Submission>(connection)?;

                let old_status = old_submission.status;
                let new_status = patch.status.unwrap_or(old_status.clone());

                // Side effects when status changes to reviewed state

                if (new_status == SubmissionStatus::Accepted
                    || new_status == SubmissionStatus::Denied
                    || new_status == SubmissionStatus::UnderConsideration)
                    && old_status != new_status
                {
                    // Send user notification
                    let level_name = levels::table
                        .filter(levels::id.eq(updated.level_id))
                        .select(levels::name)
                        .first::<String>(connection)?;

                    let (notif_type, message) = match new_status {
                        SubmissionStatus::Accepted => (
                            NotificationType::Success,
                            format!("Your submission for {:?} has been accepted!", level_name),
                        ),
                        SubmissionStatus::Denied => (
                            NotificationType::Failure,
                            format!("Your submission for {:?} has been denied.", level_name),
                        ),
                        _ => (
                            NotificationType::Info,
                            format!(
                                "Your submission for {:?} has been put under consideration.",
                                level_name
                            ),
                        ),
                    };

                    Notification::create(connection, updated.submitted_by, message, notif_type)?;
                }

                let websocket_type = if old_status != new_status {
                    match new_status {
                        SubmissionStatus::Accepted => Some("SUBMISSION_ACCEPTED"),
                        SubmissionStatus::Denied => Some("SUBMISSION_DENIED"),
                        SubmissionStatus::UnderConsideration => {
                            Some("SUBMISSION_UNDER_CONSIDERATION")
                        }
                        SubmissionStatus::UnderReview => Some("SUBMISSION_UNDER_REVIEW"),
                        _ => None,
                    }
                } else {
                    None
                };

                let completed_shift = Submission::update_user_shift(
                    connection,
                    authenticated.user_id,
                    old_status,
                    new_status,
                )?;

                Ok((updated, websocket_type, completed_shift))
            },
        )?;

        if let Some(notification_type) = websocket_type {
            WebsocketNotification::send(&notify_tx, notification_type, &result);
        }
        if let Some(completed_shift) = completed_shift {
            WebsocketNotification::send(&notify_tx, "SHIFT_COMPLETED", &completed_shift);
        }

        Ok(result)
    }

    pub fn downgrade(s: Self) -> SubmissionPatchUser {
        SubmissionPatchUser {
            mobile: s.mobile,
            ldm_id: s.ldm_id,
            video_url: s.video_url,
            raw_url: s.raw_url,
            mod_menu: s.mod_menu,
            user_notes: s.user_notes,
        }
    }
}
