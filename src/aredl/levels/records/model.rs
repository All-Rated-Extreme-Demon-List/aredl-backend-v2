use crate::aredl::records::{PublicRecordResolved, PublicRecordUnresolved};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::schema::{aredl_records, users};
use crate::users::BaseUser;
use actix_web::web;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;
use uuid::Uuid;

impl PublicRecordResolved {
    pub fn find_all_by_level(
        db: web::Data<Arc<DbAppState>>,
        level_id: Uuid,
    ) -> Result<Vec<Self>, ApiError> {
        let records = aredl_records::table
            .filter(aredl_records::level_id.eq(level_id))
            .filter(aredl_records::is_verification.eq(false))
            .inner_join(users::table.on(aredl_records::submitted_by.eq(users::id)))
            .order(aredl_records::placement_order.asc())
            .select((PublicRecordUnresolved::as_select(), BaseUser::as_select()))
            .load::<(PublicRecordUnresolved, BaseUser)>(&mut db.connection()?)?;

        let records_resolved = records
            .into_iter()
            .map(|(record, user)| Self::from_data(record, user))
            .collect();

        Ok(records_resolved)
    }

    pub fn from_data(record: PublicRecordUnresolved, user: BaseUser) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            mobile: record.mobile,
            video_url: record.video_url,
            created_at: record.created_at,
        }
    }
}
