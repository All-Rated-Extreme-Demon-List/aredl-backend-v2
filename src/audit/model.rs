use crate::{
    app_data::db::DbConnection,
    error_handler::ApiError,
    list::List,
    page_helper::{PageQuery, Paginated},
    schema::{audit_logs, users},
    users::ExtendedBaseUser,
};
use chrono::{DateTime, Utc};
use diesel::{pg::Pg, prelude::*};
use diesel_derive_enum::DbEnum;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use uuid::Uuid;

/// Represents the type of action performed in an audit log entry.
#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::AuditAction"]
#[DbValueStyle = "PascalCase"]
pub enum AuditAction {
    Create,
    Update,
    Delete,
}

/// Represents the type of entity involved in an audit log entry.
#[derive(Debug, Serialize, Deserialize, ToSchema, DbEnum, Clone, PartialEq)]
#[ExistingTypePath = "crate::schema::sql_types::AuditEntityType"]
#[DbValueStyle = "PascalCase"]
pub enum AuditEntityType {
    User,
    Level,
    Record,
    Pack,
    Role,
    Permission,
    Bounty,
    SubmissionStatus,
    Clan,
    ClanMember,
    Shift,
    LevelCustomCopy,
    Tag,
    LevelTag,
    LevelCreator,
    LevelNote,
    LevelUpdate,
    PackTier,
    RoleUser,
}

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name = audit_logs, check_for_backend(Pg))]
pub struct AuditLogEntry {
    /// Which type of action triggered this entry
    pub action_type: AuditAction,
    /// The ID of the user who performed the action (or None if automated)
    pub actor_id: Option<Uuid>,
    /// Which type of entity this entry was performed on
    pub entity_type: AuditEntityType,
    /// The ID of the entity this entry was performed on
    pub entity_id: Uuid,
    /// The ID of this entity's related parent, if applicable (e.g. the level for a custom copy)
    pub parent_entity_id: Option<Uuid>,
    /// For list-scoped entities, the list it applies to (or None for global entities i.e. users clans etc)
    pub list_id: Option<List>,
    /// Display name of the entity at the time of the action.
    pub entity_display: String,
    /// Display name of the parent at the time of the action, when available.
    pub parent_display: Option<String>,
    /// Entity before the action (excluding ignored fields).
    pub before: Option<Value>,
    /// Entity after the action (excluding ignored fields).
    pub after: Option<Value>,
    /// When this entry was created
    pub timestamp: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct AuditLogEntryResolved {
    /// Which type of action triggered this entry
    pub action_type: AuditAction,
    /// The user who performed the action (or None if automated)
    pub actor_id: Option<ExtendedBaseUser>,
    /// Which type of entity this entry was performed on
    pub entity_type: AuditEntityType,
    /// The ID of the entity this entry was performed on
    pub entity_id: Uuid,
    /// The ID of this entity's related parent, if applicable (e.g. the level for a custom copy)
    pub parent_entity_id: Option<Uuid>,
    /// For list-scoped entities, the list it applies to (or None for global entities i.e. users clans etc)
    pub list_id: Option<List>,
    /// Display name of the entity at the time of the action.
    pub entity_display: String,
    /// Display name of the parent at the time of the action, when available.
    pub parent_display: Option<String>,
    /// Entity before the action (excluding ignored fields).
    pub before: Option<Value>,
    /// Entity after the action (excluding ignored fields).
    pub after: Option<Value>,
    /// When this entry was created
    pub timestamp: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = audit_logs)]
struct NewAuditLogEntry {
    pub action_type: AuditAction,
    pub actor_id: Option<Uuid>,
    pub entity_type: AuditEntityType,
    pub entity_id: Uuid,
    pub parent_entity_id: Option<Uuid>,
    pub list_id: Option<List>,
    pub entity_display: String,
    pub parent_display: Option<String>,
    pub before: Option<Value>,
    pub after: Option<Value>,
}

pub trait Auditable: Serialize {
    const ENTITY_TYPE: AuditEntityType;
    fn entity_id(&self) -> Uuid;
    fn parent_entity_id(&self) -> Option<Uuid> {
        None
    }
    fn entity_display(&self, conn: &mut DbConnection) -> Result<String, ApiError>;
    fn parent_display(&self, _conn: &mut DbConnection) -> Result<Option<String>, ApiError> {
        Ok(None)
    }
    fn list_id(&self) -> Option<List> {
        None
    }
    fn ignored_fields() -> &'static [&'static str] {
        &[]
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct AuditLogQueryOpts {
    pub list_filter: Option<List>,
    pub actor_filter: Option<Uuid>,
    pub entity_type_filter: Option<AuditEntityType>,
    pub entity_filter: Option<Uuid>,
    pub parent_id: Option<Uuid>,
    pub action_type_filter: Option<AuditAction>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct AuditLogPage {
    /// List of found logs
    pub data: Vec<AuditLogEntryResolved>,
}

impl AuditLogEntry {
    /// Log a create/delete entry for each member added/removed to from a list.
    pub fn log_membership_changes<'a, I: Ord + 'a, T: Auditable>(
        conn: &mut DbConnection,
        actor_id: Option<Uuid>,
        before: impl IntoIterator<Item = &'a I>,
        after: impl IntoIterator<Item = &'a I>,
        entity: impl Fn(&I) -> T,
    ) -> Result<(), ApiError> {
        let before = before
            .into_iter()
            .collect::<std::collections::BTreeSet<_>>();
        let after = after.into_iter().collect::<std::collections::BTreeSet<_>>();
        for item in after.difference(&before) {
            Self::log_create(conn, actor_id, &entity(item))?;
        }
        for item in before.difference(&after) {
            Self::log_delete(conn, actor_id, &entity(item))?;
        }
        Ok(())
    }

    fn serialize_auditable<T: Auditable>(entity: &T) -> Result<Option<Value>, ApiError> {
        let mut value = serde_json::to_value(entity).map_err(|err| {
            ApiError::InternalServerError(format!("Failed to serialize audit log entity: {err}"))
        })?;

        if let Value::Object(map) = &mut value {
            for ignored_field in T::ignored_fields() {
                if map.contains_key(*ignored_field) {
                    map.remove(*ignored_field);
                }
            }
        }

        if value.as_object().is_some_and(serde_json::Map::is_empty) {
            Ok(None)
        } else {
            Ok(Some(value))
        }
    }

    /// Choose the audit action from the snapshots already read by the operation.
    pub fn log_change<T: Auditable>(
        conn: &mut DbConnection,
        actor_id: Option<Uuid>,
        before: Option<&T>,
        after: Option<&T>,
    ) -> Result<(), ApiError> {
        match (before, after) {
            (None, Some(after)) => Self::log_create(conn, actor_id, after),
            (Some(before), Some(after)) => Self::log_update(conn, actor_id, before, after),
            (Some(before), None) => Self::log_delete(conn, actor_id, before),
            (None, None) => Ok(()),
        }
    }

    pub fn log_create<T: Auditable>(
        conn: &mut DbConnection,
        actor_id: Option<Uuid>,
        entity: &T,
    ) -> Result<(), ApiError> {
        let entry = NewAuditLogEntry {
            action_type: AuditAction::Create,
            actor_id,
            entity_type: T::ENTITY_TYPE,
            entity_id: entity.entity_id(),
            parent_entity_id: entity.parent_entity_id(),
            list_id: entity.list_id(),
            entity_display: entity.entity_display(conn)?,
            parent_display: entity.parent_display(conn)?,
            before: None,
            after: Self::serialize_auditable(entity)?,
        };

        diesel::insert_into(audit_logs::table)
            .values(entry)
            .execute(conn)?;

        Ok(())
    }

    pub fn log_update<T: Auditable>(
        conn: &mut DbConnection,
        actor_id: Option<Uuid>,
        before: &T,
        after: &T,
    ) -> Result<(), ApiError> {
        let before_data = Self::serialize_auditable(before)?;
        let after_data = Self::serialize_auditable(after)?;

        if before_data == after_data {
            return Ok(());
        }

        let entry = NewAuditLogEntry {
            action_type: AuditAction::Update,
            actor_id,
            entity_type: T::ENTITY_TYPE,
            entity_id: after.entity_id(),
            parent_entity_id: after.parent_entity_id(),
            list_id: after.list_id(),
            entity_display: after.entity_display(conn)?,
            parent_display: after.parent_display(conn)?,
            before: before_data,
            after: after_data,
        };

        diesel::insert_into(audit_logs::table)
            .values(entry)
            .execute(conn)?;

        Ok(())
    }

    pub fn log_delete<T: Auditable>(
        conn: &mut DbConnection,
        actor_id: Option<Uuid>,
        entity: &T,
    ) -> Result<(), ApiError> {
        let entry = NewAuditLogEntry {
            action_type: AuditAction::Delete,
            actor_id,
            entity_type: T::ENTITY_TYPE,
            entity_id: entity.entity_id(),
            parent_entity_id: entity.parent_entity_id(),
            list_id: entity.list_id(),
            entity_display: entity.entity_display(conn)?,
            parent_display: entity.parent_display(conn)?,
            before: Self::serialize_auditable(entity)?,
            after: None,
        };

        diesel::insert_into(audit_logs::table)
            .values(entry)
            .execute(conn)?;

        Ok(())
    }

    pub fn list_all<const D: i64>(
        conn: &mut DbConnection,
        opts: &AuditLogQueryOpts,
        page_query: PageQuery<D>,
    ) -> Result<Paginated<AuditLogPage>, ApiError> {
        let build_query = || {
            let mut q = audit_logs::table.into_boxed::<Pg>();

            if let Some(list) = opts.list_filter {
                q = q.filter(audit_logs::list_id.eq(list));
            }

            if let Some(actor_id) = opts.actor_filter {
                q = q.filter(audit_logs::actor_id.eq(actor_id));
            }
            if let Some(entity_type) = &opts.entity_type_filter {
                q = q.filter(audit_logs::entity_type.eq(entity_type));
            }
            if let Some(entity_id) = opts.entity_filter {
                q = q.filter(audit_logs::entity_id.eq(entity_id));
            }
            if let Some(parent_id) = opts.parent_id {
                q = q.filter(audit_logs::parent_entity_id.eq(parent_id));
            }

            if let Some(action_type) = &opts.action_type_filter {
                q = q.filter(audit_logs::action_type.eq(action_type));
            }

            q
        };

        let total_count = build_query().count().get_result::<i64>(conn)?;

        let logs = build_query()
            .left_join(users::table)
            .order(audit_logs::timestamp.desc())
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select(<(AuditLogEntry, Option<ExtendedBaseUser>)>::as_select())
            .load::<(AuditLogEntry, Option<ExtendedBaseUser>)>(conn)?
            .into_iter()
            .map(AuditLogEntryResolved::from)
            .collect::<Vec<_>>();

        Ok(Paginated::from_data(
            page_query,
            total_count,
            AuditLogPage { data: logs },
        ))
    }
}

impl From<(AuditLogEntry, Option<ExtendedBaseUser>)> for AuditLogEntryResolved {
    fn from((entry, user): (AuditLogEntry, Option<ExtendedBaseUser>)) -> Self {
        Self {
            action_type: entry.action_type,
            actor_id: user,
            entity_type: entry.entity_type,
            entity_id: entry.entity_id,
            parent_entity_id: entry.parent_entity_id,
            list_id: entry.list_id,
            entity_display: entry.entity_display,
            parent_display: entry.parent_display,
            before: entry.before,
            after: entry.after,
            timestamp: entry.timestamp,
        }
    }
}
