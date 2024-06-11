use diesel::{Connection, delete, ExpressionMethods, insert_into, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use diesel::pg::Pg;
use crate::db;
use crate::db::DbConnection;
use crate::error_handler::ApiError;
use crate::schema::{aredl_levels_created, users};

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug)]
#[diesel(table_name=users, check_for_backend(Pg))]
pub struct Creator {
    pub id: Uuid,
    pub username: String,
    pub global_name: String,
}

impl Creator {
    pub fn find_all(level_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let creators = aredl_levels_created::table
            .filter(aredl_levels_created::level_id.eq(level_id))
            .inner_join(users::table.on(aredl_levels_created::user_id.eq(users::id)))
            .select(Creator::as_select())
            .load::<Creator>(&mut db::connection()?)?;
        Ok(creators)
    }

    pub fn add_all(level_id: Uuid, creators: Vec<Uuid>) -> Result<Vec<Uuid>, ApiError> {
        let conn = &mut db::connection()?;

        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {

            Self::add_users(level_id, creators.as_ref(), connection)?;

            let creators = aredl_levels_created::table
                .filter(aredl_levels_created::level_id.eq(level_id))
                .select(aredl_levels_created::user_id)
                .load(connection)?;

            Ok(creators)
        })?;

        Ok(result)
    }

    pub fn delete_all(level_id: Uuid, creators: Vec<Uuid>) -> Result<Vec<Uuid>, ApiError> {
        let conn = &mut db::connection()?;

        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {
            delete(aredl_levels_created::table)
                .filter(aredl_levels_created::level_id.eq(level_id))
                .filter(aredl_levels_created::user_id.eq_any(&creators))
                .execute(connection)?;

            let creators = aredl_levels_created::table
                .filter(aredl_levels_created::level_id.eq(level_id))
                .select(aredl_levels_created::user_id)
                .load(connection)?;

            Ok(creators)
        })?;

        Ok(result)
    }

    pub fn set_all(level_id: Uuid, creators: Vec<Uuid>) -> Result<Vec<Uuid>, ApiError> {
        let conn = &mut db::connection()?;

        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {
            delete(aredl_levels_created::table)
                .filter(aredl_levels_created::level_id.eq(level_id))
                .execute(connection)?;

            Self::add_users(level_id, creators.as_ref(), connection)?;

            Ok(creators)
        })?;

        Ok(result)
    }

    fn add_users(level_id: Uuid, creators: &Vec<Uuid>, connection: &mut DbConnection) -> Result<(), ApiError> {
        insert_into(aredl_levels_created::table)
            .values(
                creators.iter().map(|creator| (
                    aredl_levels_created::level_id.eq(level_id),
                    aredl_levels_created::user_id.eq(creator)
                )).collect::<Vec<_>>()
            )
            .execute(connection)?;
        Ok(())
    }
}