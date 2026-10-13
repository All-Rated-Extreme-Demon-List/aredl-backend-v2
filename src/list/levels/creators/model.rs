use crate::app_data::db::DbConnection;
use crate::audit::{AuditEntityType, AuditLogEntry};
use crate::auth::Authenticated;
use crate::error_handler::ApiError;
use crate::list::{levels::Level, List};
use crate::schema::levels_created;
use crate::schema::users;
use crate::users::{BaseUser, BaseUserWithBanLevel};
use diesel::{delete, insert_into};
use std::collections::HashSet;
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
        authenticated: &Authenticated,
    ) -> Result<Vec<Uuid>, ApiError> {
        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {
            let level = Level::find_for_update(connection, list, level_id)?;
            let added = Self::add_creators(connection, list, level_id, creators)?
                .into_iter()
                .collect::<HashSet<_>>();

            let creators: Vec<Uuid> = levels_created::table
                .filter(levels_created::list_id.eq(list))
                .filter(levels_created::level_id.eq(level_id))
                .select(levels_created::user_id)
                .load(connection)?;

            let before = creators
                .iter()
                .copied()
                .filter(|id| !added.contains(id))
                .collect::<Vec<_>>();
            AuditLogEntry::log_list_change(
                connection,
                Some(authenticated.user_id),
                &level,
                AuditEntityType::LevelCreator,
                &before,
                &creators,
            )?;

            Ok(creators)
        })?;

        Ok(result)
    }

    pub fn delete_all_creators(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        creators: &[Uuid],
        authenticated: &Authenticated,
    ) -> Result<Vec<Uuid>, ApiError> {
        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {
            let level = Level::find_for_update(connection, list, level_id)?;
            let removed = delete(levels_created::table)
                .filter(levels_created::list_id.eq(list))
                .filter(levels_created::level_id.eq(level_id))
                .filter(levels_created::user_id.eq_any(creators))
                .returning(levels_created::user_id)
                .get_results::<Uuid>(connection)?;

            let creators: Vec<Uuid> = levels_created::table
                .filter(levels_created::list_id.eq(list))
                .filter(levels_created::level_id.eq(level_id))
                .select(levels_created::user_id)
                .load(connection)?;

            let mut before = creators.clone();
            before.extend(removed);
            AuditLogEntry::log_list_change(
                connection,
                Some(authenticated.user_id),
                &level,
                AuditEntityType::LevelCreator,
                &before,
                &creators,
            )?;

            Ok(creators)
        })?;

        Ok(result)
    }

    pub fn set_all_creators(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        creators: &[Uuid],
        authenticated: &Authenticated,
    ) -> Result<Vec<Uuid>, ApiError> {
        let result = conn.transaction(|connection| -> Result<Vec<Uuid>, ApiError> {
            let level = Level::find_for_update(connection, list, level_id)?;
            let before = delete(levels_created::table)
                .filter(levels_created::list_id.eq(list))
                .filter(levels_created::level_id.eq(level_id))
                .returning(levels_created::user_id)
                .get_results::<Uuid>(connection)?;

            let after = Self::add_creators(connection, list, level_id, creators)?;
            AuditLogEntry::log_list_change(
                connection,
                Some(authenticated.user_id),
                &level,
                AuditEntityType::LevelCreator,
                &before,
                &after,
            )?;

            Ok(creators.to_vec())
        })?;

        Ok(result)
    }

    fn add_creators(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        creators: &[Uuid],
    ) -> Result<Vec<Uuid>, ApiError> {
        let added = insert_into(levels_created::table)
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
            .returning(levels_created::user_id)
            .get_results(conn)?;
        Ok(added)
    }
}
