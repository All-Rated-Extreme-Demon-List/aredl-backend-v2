use crate::app_data::db::DbConnection;
use crate::aredl::records::Record;
use crate::error_handler::ApiError;
use crate::schema::{aredl::records, users};
use crate::users::{BaseUser, ExtendedBaseUser};
use chrono::{DateTime, Utc};
use diesel::dsl::count;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(utoipa::ToSchema, Serialize, Deserialize, Debug)]
pub struct RecordQuery {
    high_extremes: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
/// A resolved record for a specific level (ommits the level field compared to ResolvedRecord).
pub struct LevelResolvedRecord {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// User who submitted the record.
    pub submitted_by: BaseUser,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Video link of the completion.
    pub video_url: String,
    /// Whether the record's video should be hidden on the website.
    pub hide_video: bool,
    /// Timestamp of when this record was achieved, used for ordering.
    pub achieved_at: DateTime<Utc>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the record was last updated.
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
/// A resolved record for a specific level (ommits the level field compared to ResolvedRecord), with an extended resolved user.
pub struct LevelResolvedRecordExtended {
    /// Internal UUID of the record.
    pub id: Uuid,
    /// User who submitted the record.
    pub submitted_by: ExtendedBaseUser,
    /// Whether the record was completed on mobile or not.
    pub mobile: bool,
    /// Video link of the completion.
    pub video_url: String,
    /// Whether the record's video should be hidden on the website.
    pub hide_video: bool,
    /// Timestamp of when this record was achieved, used for ordering.
    pub achieved_at: DateTime<Utc>,
    /// Timestamp of when the record was created (first accepted).
    pub created_at: DateTime<Utc>,
    /// Timestamp of when the record was last updated.
    pub updated_at: DateTime<Utc>,
}

impl LevelResolvedRecord {
    pub fn from_data(record: Record, user: BaseUser) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            mobile: record.mobile,
            video_url: record.video_url,
            hide_video: record.hide_video,
            achieved_at: record.achieved_at,
            updated_at: record.updated_at,
            created_at: record.created_at,
        }
    }
}

impl LevelResolvedRecordExtended {
    pub fn find_all_by_level(
        conn: &mut DbConnection,
        level_id: Uuid,
        opts: RecordQuery,
    ) -> Result<Vec<Self>, ApiError> {
        let users_high_extremes = if let Some(true) = opts.high_extremes {
            records::table
                .group_by(records::submitted_by)
                .having(count(records::id).gt(50))
                .select(records::submitted_by)
                .load::<Uuid>(conn)?
        } else {
            Vec::<Uuid>::new()
        };

        let mut query = records::table
            .filter(records::level_id.eq(level_id))
            .filter(records::is_verification.eq(false))
            .inner_join(users::table.on(records::submitted_by.eq(users::id)))
            .filter(users::ban_level.le(1))
            .into_boxed();

        if !users_high_extremes.is_empty() {
            query = query.filter(records::submitted_by.eq_any(users_high_extremes));
        }

        let records = query
            .order(records::achieved_at.asc())
            .select((Record::as_select(), ExtendedBaseUser::as_select()))
            .load::<(Record, ExtendedBaseUser)>(conn)?;

        let records_resolved = records
            .into_iter()
            .map(|(record, user)| Self::from_data(record, user))
            .collect();
        Ok(records_resolved)
    }

    pub fn from_data(record: Record, user: ExtendedBaseUser) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            mobile: record.mobile,
            video_url: record.video_url,
            hide_video: record.hide_video,
            achieved_at: record.achieved_at,
            updated_at: record.updated_at,
            created_at: record.created_at,
        }
    }
}
