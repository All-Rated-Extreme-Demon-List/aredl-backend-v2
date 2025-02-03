use std::sync::Arc;
use actix_web::web;
use diesel::{Connection, ExpressionMethods, insert_into, QueryDsl, RunQueryDsl, JoinOnDsl, SelectableHelper};
use uuid::Uuid;
use crate::db::{DbAppState, DbConnection};
use crate::error_handler::ApiError;
use crate::schema::{aredl_levels, aredl_pack_levels};
use crate::aredl::levels::BaseLevel;

impl BaseLevel {
    pub fn pack_add_all(db: web::Data<Arc<DbAppState>>, pack_id: Uuid, levels: Vec<Uuid>) -> Result<Vec<Self>, ApiError> {
        let conn = &mut db.connection()?;

        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {

            Self::add_levels(pack_id, levels.as_ref(), connection)?;

            let levels: Vec<BaseLevel> = aredl_pack_levels::table
                .filter(aredl_pack_levels::pack_id.eq(pack_id))
                .inner_join(aredl_levels::table.on(aredl_pack_levels::level_id.eq(aredl_levels::id)))
                .select(BaseLevel::as_select())
                .load(connection)?;
            Ok(levels)
        })
    }

    pub fn pack_set_all(db: web::Data<Arc<DbAppState>>, pack_id: Uuid, levels: Vec<Uuid>) -> Result<Vec<Self>, ApiError> {
        let conn = &mut db.connection()?;

        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {
            diesel::delete(aredl_pack_levels::table.filter(aredl_pack_levels::pack_id.eq(pack_id)))
                .execute(connection)?;

            Self::add_levels(pack_id, &levels, connection)?;

            let levels: Vec<BaseLevel> = aredl_pack_levels::table
                .filter(aredl_pack_levels::pack_id.eq(pack_id))
                .inner_join(aredl_levels::table.on(aredl_pack_levels::level_id.eq(aredl_levels::id)))
                .select(BaseLevel::as_select())
                .load(connection)?;
            Ok(levels)
        })
    }

    pub fn pack_delete_all(db: web::Data<Arc<DbAppState>>, pack_id: Uuid, levels: Vec<Uuid>) -> Result<Vec<Self>, ApiError> {
        let conn = &mut db.connection()?;

        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {

            Self::delete_levels(pack_id, &levels, connection)?;

            let levels: Vec<BaseLevel> = aredl_pack_levels::table
                .filter(aredl_pack_levels::pack_id.eq(pack_id))
                .inner_join(aredl_levels::table.on(aredl_pack_levels::level_id.eq(aredl_levels::id)))
                .select(BaseLevel::as_select())
                .load(connection)?;
            Ok(levels)
        })
    }

    fn add_levels(pack_id: Uuid, levels: &Vec<Uuid>, conn: &mut DbConnection) -> Result<(), ApiError> {
        insert_into(aredl_pack_levels::table)
            .values(
                levels.into_iter().map(|level| (
                    aredl_pack_levels::pack_id.eq(pack_id),
                    aredl_pack_levels::level_id.eq(level)
                )).collect::<Vec<_>>()
            )
            .execute(conn)?;
        Ok(())
    }

    pub fn delete_levels(pack_id: Uuid, levels: &Vec<Uuid>, conn: &mut DbConnection) -> Result<(), ApiError> {
            diesel::delete(
                aredl_pack_levels::table
                    .filter(aredl_pack_levels::pack_id.eq(pack_id))
                    .filter(aredl_pack_levels::level_id.eq_any(levels)),
            )
                .execute(conn)?;
            Ok(())
    }
}