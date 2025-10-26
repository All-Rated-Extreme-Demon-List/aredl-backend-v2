use crate::arepl::records::{PublicRecordResolved, PublicRecordUnresolved, PublicRecordResolvedExtended};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::schema::{arepl::records, users};
use crate::users::{BaseUser, ExtendedBaseUser};
use actix_web::web;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use diesel::dsl::count;

#[derive(utoipa::ToSchema, Serialize, Deserialize)]
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
            completion_time: record.completion_time,
        }
    }
}

impl PublicRecordResolvedExtended {
    pub fn from_data(record: PublicRecordUnresolved, user: ExtendedBaseUser) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            mobile: record.mobile,
            video_url: record.video_url,
            created_at: record.created_at,
            completion_time: record.completion_time
        }
    }
    pub fn find_all_by_level(
        db : web::Data<Arc<DbAppState>>,
        level_id: Uuid,
        opts: RecordQuery,
    ) -> Result<Vec<Self>, ApiError> {
        let mut conn = db.connection()?;
        let users_high_extremes = if let Some(true) = opts.high_extremes {
            records::table
                .group_by(records::submitted_by)
                .having(count(records::id).gt(50))
                .select(records::submitted_by)
                .load::<Uuid>(&mut conn)?
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
            .order(records::placement_order.asc())
            .select((
                PublicRecordUnresolved::as_select(),
                ExtendedBaseUser::as_select(),
            ))
            .load::<(PublicRecordUnresolved, ExtendedBaseUser)>(&mut conn)?;

        let records_resolved = records
            .into_iter()
            .map(|(record, user)| Self::from_data(record, user))
            .collect();
        Ok(records_resolved)

    }
}
