use crate::{
    app_data::db::DbConnection,
    aredl::submissions::{status::SubmissionsEnabled, *},
    auth::{Authenticated, Permission},
    error_handler::ApiError,
    notifications::WebsocketNotification,
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
use is_url::is_url;
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
    /// Video link of the completion.
    pub video_url: Option<String>,
    /// Link to the raw video file of the completion.
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
    /// Video link of the completion.
    pub video_url: Option<String>,
    /// Link to the raw video file of the completion.
    pub raw_url: Option<String>,
    /// The mod menu used in this record
    pub mod_menu: Option<String>,
    /// Any additional notes left by the submitter.
    pub user_notes: Option<String>,
    /// [Mod only] The status of the submission
    pub status: Option<SubmissionStatus>,
    /// [Mod only] Whether the record was submitted as a priority record.
    pub priority: Option<bool>,
    /// [Mod only] Internal UUID of the user who reviewed the record.
    pub reviewer_id: Option<Uuid>,
    /// [Mod only] Notes given by the reviewer when reviewing the record.
    pub reviewer_notes: Option<String>,
}

impl Submission {
    pub fn update_user_shift(
        conn: &mut DbConnection,
        notify_tx: broadcast::Sender<WebsocketNotification>,
        user_id: Uuid,
        old_status: SubmissionStatus,
        new_status: SubmissionStatus,
    ) -> Result<(), ApiError> {
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
                    let notification = WebsocketNotification {
                        notification_type: "SHIFT_COMPLETED".into(),
                        data: serde_json::to_value(&updated_shift)
                            .expect("Failed to serialize shift"),
                    };
                    if let Err(e) = notify_tx.send(notification) {
                        tracing::error!("Failed to send shift notification: {}", e);
                    }
                    diesel::update(shifts::table.filter(shifts::id.eq(shift_id)))
                        .set(shifts::status.eq(ShiftStatus::Completed))
                        .execute(conn)?;
                }
            }
        }
        Ok(())
    }
}

impl SubmissionPatchUser {
    pub fn patch(
        patch: Self,
        id: Uuid,
        conn: &mut DbConnection,
        authenticated: Authenticated,
    ) -> Result<Submission, ApiError> {
        let user = authenticated.user_id;

        if patch == Self::default() {
            return Err(ApiError::new(400, "No changes were provided!"));
        }

        if let Some(video_url) = patch.video_url.as_ref() {
            if !is_url(video_url) {
                return Err(ApiError::new(
                    400,
                    "Completion video link is not a valid URL!",
                ));
            }
        }

        if let Some(raw_url) = patch.raw_url.as_ref() {
            if !is_url(raw_url) {
                return Err(ApiError::new(400, "Raw footage link is not a valid URL!"));
            }
        }

        let submitter_ban = users::table
            .filter(users::id.eq(user))
            .select(users::ban_level)
            .first::<i32>(conn)?;

        if submitter_ban >= 2 {
            return Err(ApiError::new(
                403,
                "You have been banned from submitting records.",
            ));
        }

        let old_submission = submissions::table
            .filter(submissions::id.eq(id))
            .select(Submission::as_select())
            .first::<Submission>(conn)?;

        if old_submission.submitted_by != user {
            return Err(ApiError::new(
                403,
                "You can only edit your own submissions.",
            ));
        }

        match old_submission.status {
            SubmissionStatus::Claimed
            | SubmissionStatus::UnderConsideration
            | SubmissionStatus::UnderReview => {
                return Err(ApiError::new(
                    409,
                    "This submission is currently being reviewed and cannot be edited.",
                ));
            }
            _ => {}
        }

        if !SubmissionsEnabled::is_enabled(conn)?
            && old_submission.status != SubmissionStatus::Pending
        {
            return Err(ApiError::new(
                400,
                "Submissions are currently closed. You can only edit pending submissions.",
            ));
        }

        if patch.raw_url.is_some() {
            let raw_footage = patch.raw_url.clone().or(old_submission.raw_url.clone());

            let level_info = levels::table
                .filter(levels::id.eq(old_submission.level_id))
                .select((levels::legacy, levels::position))
                .first::<(bool, i32)>(conn)
                .optional()?;

            match level_info {
                None => {
                    return Err(ApiError::new(
                        404,
                        "Could not find the level for this submission.",
                    ))
                }
                Some((is_legacy, pos)) => {
                    if is_legacy {
                        return Err(ApiError::new(
                            400,
                            "This level is on the legacy list and is not accepting records!",
                        ));
                    }
                    if pos < 400 && raw_footage.is_none() {
                        return Err(ApiError::new(
                            400,
                            "This level is top 400 and requires raw footage!",
                        ));
                    }
                }
            }
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
        patch: Self,
        id: Uuid,
        conn: &mut DbConnection,
        authenticated: Authenticated,
        notify_tx: broadcast::Sender<WebsocketNotification>,
    ) -> Result<Submission, ApiError> {
        if patch == Self::default() {
            return Err(ApiError::new(400, "No changes were provided!"));
        }

        if !authenticated.has_permission(conn, Permission::SubmissionReview)? {
            return Err(ApiError::new(
                403,
                "You do not have permission to review submissions.",
            ));
        }

        if let Some(video_url) = patch.video_url.as_ref() {
            if !is_url(video_url) {
                return Err(ApiError::new(
                    400,
                    "Completion video link is not a valid URL!",
                ));
            }
        }

        if let Some(raw_url) = patch.raw_url.as_ref() {
            if !is_url(raw_url) {
                return Err(ApiError::new(400, "Raw footage link is not a valid URL!"));
            }
        }

        let old_submission = submissions::table
            .filter(submissions::id.eq(id))
            .select(Submission::as_select())
            .first::<Submission>(conn)?;

        if old_submission.submitted_by == authenticated.user_id {
            return SubmissionPatchUser::patch(
                SubmissionPatchMod::downgrade(patch),
                id,
                conn,
                authenticated,
            );
        }

        let result = conn.transaction(|connection| -> Result<Submission, ApiError> {
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

            if (new_status == SubmissionStatus::Accepted
                || new_status == SubmissionStatus::Denied
                || new_status == SubmissionStatus::UnderConsideration)
                && old_status != new_status
            {
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

                let ws_type = match new_status {
                    SubmissionStatus::Accepted => "SUBMISSION_ACCEPTED",
                    SubmissionStatus::Denied => "SUBMISSION_DENIED",
                    _ => "SUBMISSION_UNDER_CONSIDERATION",
                };

                let notification = WebsocketNotification {
                    notification_type: ws_type.into(),
                    data: serde_json::to_value(&updated).expect("Failed to serialize submission"),
                };
                let _ = notify_tx.send(notification);
            }

            Submission::update_user_shift(
                connection,
                notify_tx.clone(),
                authenticated.user_id,
                old_status,
                new_status,
            )?;

            Ok(updated)
        })?;

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
