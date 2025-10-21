use crate::aredl::records::{
    PublicRecordResolved, PublicRecordResolvedWithCountry, PublicRecordUnresolved,
};
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::{aredl::records, users};
use crate::users::{BaseUser, BaseUserWithCountry};
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;

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

impl PublicRecordResolvedWithCountry {
    pub fn find_all_by_level(
        conn: &mut DbConnection,
        level_id: Uuid,
    ) -> Result<Vec<Self>, ApiError> {
        let records = records::table
            .filter(records::level_id.eq(level_id))
            .filter(records::is_verification.eq(false))
            .inner_join(users::table.on(records::submitted_by.eq(users::id)))
            .filter(users::ban_level.le(1))
            .order(records::placement_order.asc())
            .select((
                PublicRecordUnresolved::as_select(),
                BaseUserWithCountry::as_select(),
            ))
            .load::<(PublicRecordUnresolved, BaseUserWithCountry)>(conn)?;

        let records_resolved = records
            .into_iter()
            .map(|(record, user)| Self::from_data(record, user))
            .collect();

        Ok(records_resolved)
    }

    pub fn from_data(record: PublicRecordUnresolved, user: BaseUserWithCountry) -> Self {
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
