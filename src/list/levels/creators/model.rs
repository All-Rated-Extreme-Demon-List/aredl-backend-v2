use crate::app_data::db::DbConnection;
use crate::error_handler::ApiError;
use crate::list::List;
use crate::schema::levels_created;
use crate::schema::users;
use crate::users::{BaseUser, BaseUserWithBanLevel};
use diesel::{delete, insert_into};
use uuid::Uuid;

use diesel::prelude::*;
impl BaseUser {
    pub fn find_all_creators(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
    ) -> Result<Vec<Self>, ApiError> {
        let creators = levels_created::table
            .filter(levels_created::list_id.eq(list))
            .filter(levels_created::level_id.eq(level_id))
            .inner_join(users::table)
            .select(BaseUserWithBanLevel::as_select())
            .load::<BaseUserWithBanLevel>(conn)?;
        let creators = creators
            .into_iter()
            .map(BaseUser::from_base_user_with_ban_level)
            .collect();
        Ok(creators)
    }

    pub fn add_all_creators(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        creators: &[Uuid],
    ) -> Result<Vec<Uuid>, ApiError> {
        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {
            Self::add_creators(connection, list, level_id, creators)?;

            let creators = levels_created::table
                .filter(levels_created::list_id.eq(list))
                .filter(levels_created::level_id.eq(level_id))
                .select(levels_created::user_id)
                .load(connection)?;

            Ok(creators)
        })?;

        Ok(result)
    }

    pub fn delete_all_creators(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        creators: &[Uuid],
    ) -> Result<Vec<Uuid>, ApiError> {
        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {
            delete(levels_created::table)
                .filter(levels_created::list_id.eq(list))
                .filter(levels_created::level_id.eq(level_id))
                .filter(levels_created::user_id.eq_any(creators))
                .execute(connection)?;

            let creators = levels_created::table
                .filter(levels_created::list_id.eq(list))
                .filter(levels_created::level_id.eq(level_id))
                .select(levels_created::user_id)
                .load(connection)?;

            Ok(creators)
        })?;

        Ok(result)
    }

    pub fn set_all_creators(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        creators: &[Uuid],
    ) -> Result<Vec<Uuid>, ApiError> {
        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {
            delete(levels_created::table)
                .filter(levels_created::list_id.eq(list))
                .filter(levels_created::level_id.eq(level_id))
                .execute(connection)?;

            Self::add_creators(connection, list, level_id, creators)?;

            Ok(creators.to_vec())
        })?;

        Ok(result)
    }

    fn add_creators(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        creators: &[Uuid],
    ) -> Result<(), ApiError> {
        insert_into(levels_created::table)
            .values(
                creators
                    .iter()
                    .map(|creator| {
                        (
                            levels_created::list_id.eq(list),
                            levels_created::level_id.eq(level_id),
                            levels_created::user_id.eq(creator),
                        )
                    })
                    .collect::<Vec<_>>(),
            )
            .execute(conn)?;
        Ok(())
    }
}
