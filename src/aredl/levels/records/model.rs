use crate::aredl::records::{Record, RecordResolved, RecordUnresolved};
use crate::db::DbAppState;
use crate::error_handler::ApiError;
use crate::schema::{aredl_records, users};
use crate::users::BaseUser;
use actix_web::web;
use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use std::sync::Arc;
use uuid::Uuid;

impl Record {
    pub fn find_all(db: web::Data<Arc<DbAppState>>, level_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let records = aredl_records::table
            .filter(aredl_records::level_id.eq(level_id))
            .select(Record::as_select())
            .order(aredl_records::is_verification.desc())
            .then_order_by(aredl_records::placement_order.asc())
            .load::<Self>(&mut db.connection()?)?;
        Ok(records)
    }

    pub fn find(
        db: web::Data<Arc<DbAppState>>,
        level_id: Uuid,
        record_id: Uuid,
    ) -> Result<Self, ApiError> {
        let record = aredl_records::table
            .filter(aredl_records::level_id.eq(level_id))
            .filter(aredl_records::id.eq(record_id))
            .select(Record::as_select())
            .first::<Self>(&mut db.connection()?)?;
        Ok(record)
    }
}

impl RecordResolved {
    pub fn find_all(db: web::Data<Arc<DbAppState>>, level_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let records = aredl_records::table
            .filter(aredl_records::level_id.eq(level_id))
            .filter(aredl_records::is_verification.eq(false))
            .inner_join(users::table.on(aredl_records::submitted_by.eq(users::id)))
            .order(aredl_records::placement_order.asc())
            .select((RecordUnresolved::as_select(), BaseUser::as_select()))
            .load::<(RecordUnresolved, BaseUser)>(&mut db.connection()?)?;

        let records_resolved = records
            .into_iter()
            .map(|(record, user)| Self::from_data(record, user))
            .collect();

        Ok(records_resolved)
    }

    fn from_data(record: RecordUnresolved, user: BaseUser) -> Self {
        Self {
            id: record.id,
            submitted_by: user,
            mobile: record.mobile,
            video_url: record.video_url,
            created_at: record.created_at,
        }
    }
}
