use std::sync::Arc;
use actix_web::web;
use diesel::{Connection, ExpressionMethods, insert_into, QueryDsl, RunQueryDsl};
use uuid::Uuid;
use serde::Serialize;
use utoipa::ToSchema;
use crate::db::{DbAppState, DbConnection};
use crate::error_handler::ApiError;
use crate::schema::aredl_pack_levels;

#[derive(Serialize, ToSchema)]
pub struct BasePackLevel {
    /// UUID of the level.
    pub id: Uuid
}

impl BasePackLevel {
    pub fn add_all(db: web::Data<Arc<DbAppState>>, pack_id: Uuid, levels: Vec<Uuid>) -> Result<Vec<Self>, ApiError> {
        let conn = &mut db.connection()?;

        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {

            Self::add_levels(pack_id, levels.as_ref(), connection)?;

            let levels = aredl_pack_levels::table
                .filter(aredl_pack_levels::pack_id.eq(pack_id))
                .select(aredl_pack_levels::level_id)
                .load(connection)?
                .into_iter()
                .map(|id| Self {id})
                .collect::<Vec<Self>>();

            Ok(levels)
        })
    }

    pub fn set_all(db: web::Data<Arc<DbAppState>>, pack_id: Uuid, levels: Vec<Uuid>) -> Result<Vec<Self>, ApiError> {
        let conn = &mut db.connection()?;

        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {
            diesel::delete(aredl_pack_levels::table.filter(aredl_pack_levels::pack_id.eq(pack_id)))
                .execute(connection)?;

            Self::add_levels(pack_id, &levels, connection)?;

            let levels = aredl_pack_levels::table
                .filter(aredl_pack_levels::pack_id.eq(pack_id))
                .select(aredl_pack_levels::level_id)
                .load(connection)?
                .into_iter()
                .map(|id| Self { id })
                .collect::<Vec<Self>>();

            Ok(levels)
        })
    }

    pub fn delete_all(db: web::Data<Arc<DbAppState>>, pack_id: Uuid, levels: Vec<Uuid>) -> Result<Vec<Self>, ApiError> {
        let conn = &mut db.connection()?;

        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {

            Self::delete_levels(pack_id, &levels, connection)?;

            let levels = aredl_pack_levels::table
                .filter(aredl_pack_levels::pack_id.eq(pack_id))
                .select(aredl_pack_levels::level_id)
                .load(connection)?
                .into_iter()
                .map(|id| Self { id })
                .collect::<Vec<Self>>();

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