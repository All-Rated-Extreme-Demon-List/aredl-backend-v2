use std::sync::Arc;
use actix_web::web;
use diesel::{Connection, delete, ExpressionMethods, insert_into, JoinOnDsl, QueryDsl, RunQueryDsl, SelectableHelper};
use uuid::Uuid;
use crate::db::{DbAppState, DbConnection};
use crate::error_handler::ApiError;
use crate::users::BaseUser;
use crate::schema::{aredl_levels_created, users};

impl BaseUser {
    pub fn find_all_creators(db: web::Data<Arc<DbAppState>>, level_id: Uuid) -> Result<Vec<Self>, ApiError> {
        let creators = aredl_levels_created::table
            .filter(aredl_levels_created::level_id.eq(level_id))
            .inner_join(users::table.on(aredl_levels_created::user_id.eq(users::id)))
            .select(BaseUser::as_select())
            .load::<BaseUser>(&mut db.connection()?)?;
        Ok(creators)
    }

    pub fn add_all_creators(db: web::Data<Arc<DbAppState>>, level_id: Uuid, creators: Vec<Uuid>) -> Result<Vec<Uuid>, ApiError> {
        let conn = &mut db.connection()?;

        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {

            Self::add_creators(level_id, creators.as_ref(), connection)?;

            let creators = aredl_levels_created::table
                .filter(aredl_levels_created::level_id.eq(level_id))
                .select(aredl_levels_created::user_id)
                .load(connection)?;

            Ok(creators)
        })?;

        Ok(result)
    }

    pub fn delete_all_creators(db: web::Data<Arc<DbAppState>>, level_id: Uuid, creators: Vec<Uuid>) -> Result<Vec<Uuid>, ApiError> {
        let conn = &mut db.connection()?;

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

    pub fn set_all_creators(db: web::Data<Arc<DbAppState>>, level_id: Uuid, creators: Vec<Uuid>) -> Result<Vec<Uuid>, ApiError> {
        let conn = &mut db.connection()?;

        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {
            delete(aredl_levels_created::table)
                .filter(aredl_levels_created::level_id.eq(level_id))
                .execute(connection)?;

            Self::add_creators(level_id, creators.as_ref(), connection)?;

            Ok(creators)
        })?;

        Ok(result)
    }

    fn add_creators(level_id: Uuid, creators: &Vec<Uuid>, connection: &mut DbConnection) -> Result<(), ApiError> {
        insert_into(aredl_levels_created::table)
            .values(
                creators.into_iter().map(|creator| (
                    aredl_levels_created::level_id.eq(level_id),
                    aredl_levels_created::user_id.eq(creator)
                )).collect::<Vec<_>>()
            )
            .execute(connection)?;
        Ok(())
    }
}