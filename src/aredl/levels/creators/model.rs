use crate::db::{DbAppState, DbConnection};
use crate::error_handler::ApiError;
use crate::schema::aredl::levels_created;
use crate::schema::users;
use crate::users::{BaseUser, BaseUserWithBanLevel};
use actix_web::web;
use diesel::{
    delete, insert_into, Connection, ExpressionMethods, JoinOnDsl, QueryDsl, RunQueryDsl,
    SelectableHelper,
};
use std::sync::Arc;
use uuid::Uuid;

impl BaseUser {
    pub fn aredl_find_all_creators(
        db: web::Data<Arc<DbAppState>>,
        level_id: Uuid,
    ) -> Result<Vec<Self>, ApiError> {
        let creators = levels_created::table
            .filter(levels_created::level_id.eq(level_id))
            .inner_join(users::table.on(levels_created::user_id.eq(users::id)))
            .select(BaseUserWithBanLevel::as_select())
            .load::<BaseUserWithBanLevel>(&mut db.connection()?)?;
        let creators = creators
            .into_iter()
            .map(|creator| BaseUser::from_base_user_with_ban_level(creator))
            .collect();
        Ok(creators)
    }

    pub fn aredl_add_all_creators(
        db: web::Data<Arc<DbAppState>>,
        level_id: Uuid,
        creators: Vec<Uuid>,
    ) -> Result<Vec<Uuid>, ApiError> {
        let conn = &mut db.connection()?;

        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {
            Self::aredl_add_creators(level_id, creators.as_ref(), connection)?;

            let creators = levels_created::table
                .filter(levels_created::level_id.eq(level_id))
                .select(levels_created::user_id)
                .load(connection)?;

            Ok(creators)
        })?;

        Ok(result)
    }

    pub fn aredl_delete_all_creators(
        db: web::Data<Arc<DbAppState>>,
        level_id: Uuid,
        creators: Vec<Uuid>,
    ) -> Result<Vec<Uuid>, ApiError> {
        let conn = &mut db.connection()?;

        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {
            delete(levels_created::table)
                .filter(levels_created::level_id.eq(level_id))
                .filter(levels_created::user_id.eq_any(&creators))
                .execute(connection)?;

            let creators = levels_created::table
                .filter(levels_created::level_id.eq(level_id))
                .select(levels_created::user_id)
                .load(connection)?;

            Ok(creators)
        })?;

        Ok(result)
    }

    pub fn aredl_set_all_creators(
        db: web::Data<Arc<DbAppState>>,
        level_id: Uuid,
        creators: Vec<Uuid>,
    ) -> Result<Vec<Uuid>, ApiError> {
        let conn = &mut db.connection()?;

        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {
            delete(levels_created::table)
                .filter(levels_created::level_id.eq(level_id))
                .execute(connection)?;

            Self::aredl_add_creators(level_id, creators.as_ref(), connection)?;

            Ok(creators)
        })?;

        Ok(result)
    }

    fn aredl_add_creators(
        level_id: Uuid,
        creators: &Vec<Uuid>,
        connection: &mut DbConnection,
    ) -> Result<(), ApiError> {
        insert_into(levels_created::table)
            .values(
                creators
                    .into_iter()
                    .map(|creator| {
                        (
                            levels_created::level_id.eq(level_id),
                            levels_created::user_id.eq(creator),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .execute(connection)?;
        Ok(())
    }
}
