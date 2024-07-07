use diesel::{Connection, ExpressionMethods, insert_into, QueryDsl, RunQueryDsl};
use uuid::Uuid;
use crate::db;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::{aredl_pack_levels};

pub struct Level {
    pub id: Uuid
}

impl Level {
    pub fn add_all(pack_id: Uuid, levels: Vec<Uuid>) -> Result<Vec<Self>, ApiError> {
        let conn = &mut db::connection()?;

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
}