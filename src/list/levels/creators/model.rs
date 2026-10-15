use crate::app_data::db::DbConnection;
use crate::audit::{AuditEntityType, AuditLogEntry, Auditable};
use crate::auth::Authenticated;
use crate::error_handler::ApiError;
use crate::list::List;
use crate::schema::{levels, levels_created, users};
use crate::users::{BaseUser, BaseUserWithAccountStatus};
use diesel::{delete, insert_into};
use serde::Serialize;
use uuid::Uuid;

use diesel::prelude::*;

#[derive(Serialize)]
struct LevelCreator {
    list_id: List,
    level_id: Uuid,
    user_id: Uuid,
}

impl Auditable for LevelCreator {
    const ENTITY_TYPE: AuditEntityType = AuditEntityType::LevelCreator;

    fn entity_id(&self) -> Uuid {
        self.user_id
    }

    fn parent_entity_id(&self) -> Option<Uuid> {
        Some(self.level_id)
    }

    fn list_id(&self) -> Option<List> {
        Some(self.list_id)
    }

    fn entity_display(&self, conn: &mut DbConnection) -> Result<String, ApiError> {
        Ok(users::table
            .find(self.user_id)
            .select(users::global_name)
            .first(conn)?)
    }

    fn parent_display(&self, conn: &mut DbConnection) -> Result<Option<String>, ApiError> {
        Ok(Some(
            levels::table
                .filter(levels::list_id.eq(self.list_id))
                .find(self.level_id)
                .select(levels::name)
                .first(conn)?,
        ))
    }

    // data is already stored in the id fields
    fn ignored_fields() -> &'static [&'static str] {
        &["list_id", "level_id", "user_id"]
    }
}

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
            .select(BaseUserWithAccountStatus::as_select())
            .load::<BaseUserWithAccountStatus>(conn)?;
        let creators = creators
            .into_iter()
            .map(BaseUser::from_base_user_with_account_status)
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
            let added = Self::add_creators(connection, list, level_id, creators)?;

            let creators: Vec<Uuid> = levels_created::table
                .filter(levels_created::list_id.eq(list))
                .filter(levels_created::level_id.eq(level_id))
                .select(levels_created::user_id)
                .load(connection)?;

            AuditLogEntry::log_membership_changes(
                connection,
                Some(authenticated.user_id),
                &[],
                &added,
                |user_id| LevelCreator {
                    list_id: list,
                    level_id,
                    user_id: *user_id,
                },
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

            AuditLogEntry::log_membership_changes(
                connection,
                Some(authenticated.user_id),
                &removed,
                &[],
                |user_id| LevelCreator {
                    list_id: list,
                    level_id,
                    user_id: *user_id,
                },
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
            let before = delete(levels_created::table)
                .filter(levels_created::list_id.eq(list))
                .filter(levels_created::level_id.eq(level_id))
                .returning(levels_created::user_id)
                .get_results::<Uuid>(connection)?;

            let after = Self::add_creators(connection, list, level_id, creators)?;
            AuditLogEntry::log_membership_changes(
                connection,
                Some(authenticated.user_id),
                &before,
                &after,
                |user_id| LevelCreator {
                    list_id: list,
                    level_id,
                    user_id: *user_id,
                },
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
