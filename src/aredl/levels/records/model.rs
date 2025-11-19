use crate::app_data::db::DbConnection;
use crate::aredl::records::{
    PublicRecordResolved, PublicRecordResolvedExtended, PublicRecordUnresolved,
};
use crate::error_handler::ApiError;
use crate::schema::{aredl::records, users};
use crate::users::{BaseUser, ExtendedBaseUser};
use diesel::dsl::count;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(utoipa::ToSchema, Serialize, Deserialize, Debug)]
pub struct RecordQuery {
    high_extremes: Option<bool>,
}

impl PublicRecordResolved {
    pub fn from_data(record: PublicRecordUnresolved, user: BaseUser) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            mobile: record.mobile,
            video_url: record.video_url,
            created_at: record.created_at,
            hide_video: record.hide_video,
        }
    }
}

impl PublicRecordResolvedExtended {
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
            .order(records::created_at.asc())
            .select((
                PublicRecordUnresolved::as_select(),
                ExtendedBaseUser::as_select(),
            ))
            .load::<(PublicRecordUnresolved, ExtendedBaseUser)>(conn)?;

        let records_resolved = records
            .into_iter()
            .map(|(record, user)| Self::from_data(record, user))
            .collect();
        Ok(records_resolved)
    }

    pub fn from_data(record: PublicRecordUnresolved, user: ExtendedBaseUser) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            mobile: record.mobile,
            video_url: record.video_url,
            created_at: record.created_at,
            hide_video: record.hide_video,
        }
    }
}
