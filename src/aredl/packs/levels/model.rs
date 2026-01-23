use crate::aredl::levels::BaseLevel;
use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::{aredl::levels, aredl::pack_levels};
use diesel::{
    insert_into, Connection, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper,
};
use uuid::Uuid;

impl BaseLevel {
    pub fn pack_add_all(
        conn: &mut DbConnection,
        pack_id: Uuid,
        levels: Vec<Uuid>,
    ) -> Result<Vec<Self>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {
            Self::add_levels(pack_id, levels.as_ref(), connection)?;

            let levels: Vec<BaseLevel> = pack_levels::table
                .filter(pack_levels::pack_id.eq(pack_id))
                .inner_join(levels::table.on(pack_levels::level_id.eq(levels::id)))
                .select(BaseLevel::as_select())
                .load(connection)?;
            Ok(levels)
        })
    }

    pub fn pack_set_all(
        conn: &mut DbConnection,
        pack_id: Uuid,
        levels: Vec<Uuid>,
    ) -> Result<Vec<Self>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {
            diesel::delete(pack_levels::table.filter(pack_levels::pack_id.eq(pack_id)))
                .execute(connection)?;

            Self::add_levels(pack_id, &levels, connection)?;

            let levels: Vec<BaseLevel> = pack_levels::table
                .filter(pack_levels::pack_id.eq(pack_id))
                .inner_join(levels::table.on(pack_levels::level_id.eq(levels::id)))
                .select(BaseLevel::as_select())
                .load(connection)?;
            Ok(levels)
        })
    }

    pub fn pack_delete_all(
        conn: &mut DbConnection,
        pack_id: Uuid,
        levels: Vec<Uuid>,
    ) -> Result<Vec<Self>, ApiError> {
        conn.transaction(move |connection| -> Result<Vec<Self>, ApiError> {
            Self::delete_levels(pack_id, &levels, connection)?;

            let levels: Vec<BaseLevel> = pack_levels::table
                .filter(pack_levels::pack_id.eq(pack_id))
                .inner_join(levels::table.on(pack_levels::level_id.eq(levels::id)))
                .select(BaseLevel::as_select())
                .load(connection)?;
            Ok(levels)
        })
    }

    fn add_levels(
        pack_id: Uuid,
        levels: &Vec<Uuid>,
        conn: &mut DbConnection,
    ) -> Result<(), ApiError> {
        insert_into(pack_levels::table)
            .values(
                levels
                    .into_iter()
                    .map(|level| {
                        (
                            pack_levels::pack_id.eq(pack_id),
                            pack_levels::level_id.eq(level),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .execute(conn)?;
        Ok(())
    }

    pub fn delete_levels(
        pack_id: Uuid,
        levels: &Vec<Uuid>,
        conn: &mut DbConnection,
    ) -> Result<(), ApiError> {
        diesel::delete(
            pack_levels::table
                .filter(pack_levels::pack_id.eq(pack_id))
                .filter(pack_levels::level_id.eq_any(levels)),
        )
        .execute(conn)?;
        Ok(())
    }
}
