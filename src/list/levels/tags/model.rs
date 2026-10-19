use crate::app_data::db::DbConnection;
use crate::audit::{AuditEntityType, AuditLogEntry, Auditable};
use crate::auth::Authenticated;
use crate::error_handler::ApiError;
use crate::list::List;
use crate::schema::{level_tags, levels, tags};
use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
pub(super) struct LevelTag {
    list_id: List,
    level_id: Uuid,
    tag_id: Uuid,
}

impl Auditable for LevelTag {
    const ENTITY_TYPE: AuditEntityType = AuditEntityType::LevelTag;

    fn entity_id(&self) -> Uuid {
        self.tag_id
    }

    fn parent_entity_id(&self) -> Option<Uuid> {
        Some(self.level_id)
    }

    fn list_id(&self) -> Option<List> {
        Some(self.list_id)
    }

    fn entity_display(&self, conn: &mut DbConnection) -> Result<String, ApiError> {
        Ok(tags::table
            .filter(tags::list_id.eq(self.list_id))
            .find(self.tag_id)
            .select(tags::name)
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

    fn ignored_fields() -> &'static [&'static str] {
        &["list_id", "level_id", "tag_id"]
    }
}

impl LevelTag {
    pub fn set_all_level(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        tag_ids: &[Uuid],
        authenticated: &Authenticated,
    ) -> Result<Vec<Uuid>, ApiError> {
        conn.transaction(|conn| {
            let before = diesel::delete(
                level_tags::table
                    .filter(level_tags::list_id.eq(list))
                    .filter(level_tags::level_id.eq(level_id)),
            )
            .returning(level_tags::tag_id)
            .get_results::<Uuid>(conn)?;
            let mut after = Self::add_level_tags(conn, list, level_id, tag_ids)?;
            after.sort_unstable();

            AuditLogEntry::log_membership_changes(
                conn,
                Some(authenticated.user_id),
                &before,
                &after,
                |tag_id| LevelTag {
                    list_id: list,
                    level_id,
                    tag_id: *tag_id,
                },
            )?;
            Ok(after)
        })
    }

    pub fn add_all_level(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        tag_ids: &[Uuid],
        authenticated: &Authenticated,
    ) -> Result<Vec<Uuid>, ApiError> {
        conn.transaction(|conn| {
            let added = Self::add_level_tags(conn, list, level_id, tag_ids)?;
            let current = level_tags::table
                .filter(level_tags::list_id.eq(list))
                .filter(level_tags::level_id.eq(level_id))
                .order(level_tags::tag_id)
                .select(level_tags::tag_id)
                .load(conn)?;

            AuditLogEntry::log_membership_changes(
                conn,
                Some(authenticated.user_id),
                &[],
                &added,
                |tag_id| LevelTag {
                    list_id: list,
                    level_id,
                    tag_id: *tag_id,
                },
            )?;
            Ok(current)
        })
    }

    pub fn delete_all_level(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        tag_ids: &[Uuid],
        authenticated: &Authenticated,
    ) -> Result<Vec<Uuid>, ApiError> {
        conn.transaction(|conn| {
            let removed = diesel::delete(
                level_tags::table
                    .filter(level_tags::list_id.eq(list))
                    .filter(level_tags::level_id.eq(level_id))
                    .filter(level_tags::tag_id.eq_any(tag_ids)),
            )
            .returning(level_tags::tag_id)
            .get_results::<Uuid>(conn)?;
            let current = level_tags::table
                .filter(level_tags::list_id.eq(list))
                .filter(level_tags::level_id.eq(level_id))
                .order(level_tags::tag_id)
                .select(level_tags::tag_id)
                .load(conn)?;

            AuditLogEntry::log_membership_changes(
                conn,
                Some(authenticated.user_id),
                &removed,
                &[],
                |tag_id| LevelTag {
                    list_id: list,
                    level_id,
                    tag_id: *tag_id,
                },
            )?;
            Ok(current)
        })
    }

    fn add_level_tags(
        conn: &mut DbConnection,
        list: List,
        level_id: Uuid,
        tag_ids: &[Uuid],
    ) -> Result<Vec<Uuid>, ApiError> {
        let level_tags_list = tag_ids
            .iter()
            .map(|tag_id| {
                (
                    level_tags::list_id.eq(list),
                    level_tags::level_id.eq(level_id),
                    level_tags::tag_id.eq(*tag_id),
                )
            })
            .collect::<Vec<_>>();
        Ok(diesel::insert_into(level_tags::table)
            .values(level_tags_list)
            .on_conflict_do_nothing()
            .returning(level_tags::tag_id)
            .get_results(conn)?)
    }
}
