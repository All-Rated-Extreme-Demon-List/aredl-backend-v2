use crate::{
    app_data::db::DbConnection,
    error_handler::ApiError,
    page_helper::{PageQuery, Paginated},
    schema::audit_logs,
};
use chrono::{DateTime, Utc};
use diesel::{
    pg::Pg, ExpressionMethods as _, Insertable, QueryDsl as _, Queryable, RunQueryDsl as _,
    Selectable, SelectableHelper as _,
};
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
    CustomCopy,
    SubmissionStatus,
    Clan,
    Shift,
}

#[derive(Serialize, Deserialize, Selectable, Queryable, Debug, ToSchema)]
#[diesel(table_name = audit_logs, check_for_backend(Pg))]
pub struct AuditLogEntry {
    pub action_type: AuditAction,
    pub actor_id: Option<Uuid>,
    pub entity_type: AuditEntityType,
    pub entity_id: Uuid,
    pub diff: Option<Value>,
    pub timestamp: DateTime<Utc>,
}

#[derive(Insertable)]
#[diesel(table_name = audit_logs)]
struct NewAuditLogEntry {
    pub action_type: AuditAction,
    pub actor_id: Option<Uuid>,
    pub entity_type: AuditEntityType,
    pub entity_id: Uuid,
    pub diff: Option<Value>,
}

pub trait Auditable: Serialize {
    const ENTITY_TYPE: AuditEntityType;
    fn entity_id(&self) -> Uuid;
    fn redacted_fields() -> &'static [&'static str] {
        &[]
    }
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct AuditLogQueryOpts {
    pub actor_filter: Option<Uuid>,
    pub entity_type_filter: Option<AuditEntityType>,
    pub entity_filter: Option<Uuid>,
    pub action_type_filter: Option<AuditAction>,
}

#[derive(Serialize, Deserialize, Debug, ToSchema)]
pub struct AuditLogPage {
    /// List of found logs
    pub data: Vec<AuditLogEntry>,
}

impl AuditLogEntry {
    fn serialize_auditable<T: Auditable>(entity: &T) -> Result<Value, ApiError> {
        let mut value = serde_json::to_value(entity).map_err(|err| {
            ApiError::InternalServerError(format!("Failed to serialize audit log entity: {err}"))
        })?;

        if let Value::Object(map) = &mut value {
            for redacted_field in T::redacted_fields() {
                if map.contains_key(*redacted_field) {
                    map.remove(*redacted_field);
                }
            }
        }

        Ok(value)
    }

    fn generate_update_diff<T: Auditable>(before: &T, after: &T) -> Result<Value, ApiError> {
        let before_value = Self::serialize_auditable(before)?;
        let after_value = Self::serialize_auditable(after)?;

        let (Some(before_obj), Some(after_obj)) =
            (before_value.as_object(), after_value.as_object())
        else {
            return Err(ApiError::InternalServerError("Invalid diff value(s)"));
        };

        let mut diff = serde_json::Map::new();
        let keys = before_obj
            .keys()
            .chain(after_obj.keys())
            .collect::<std::collections::BTreeSet<_>>();

        for key in keys {
            let before_value = before_obj.get(key);
            let after_value = after_obj.get(key);

            match (before_value, after_value) {
                (Some(before), Some(after)) if before != after => {
                    diff.insert(key.clone(), after.clone());
                }
                (None, Some(after)) => {
                    diff.insert(key.clone(), after.clone());
                }
                (Some(_), None) => {
                    diff.insert(key.clone(), Value::Null);
                }
                _ => {}
            }
        }

        Ok(Value::Object(diff))
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
            diff: Some(Self::serialize_auditable(entity)?),
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
        let diff = Self::generate_update_diff(before, after)?;

        if diff.as_object().is_none_or(serde_json::Map::is_empty) {
            return Ok(());
        }

        let entry = NewAuditLogEntry {
            action_type: AuditAction::Update,
            actor_id,
            entity_type: T::ENTITY_TYPE,
            entity_id: after.entity_id(),
            diff: Some(diff),
        };

        diesel::insert_into(audit_logs::table)
            .values(entry)
            .execute(conn)?;

        Ok(())
    }

    #[expect(dead_code, reason = "Only users is implemented rn")]
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
            diff: None,
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

            if let Some(actor_id) = opts.actor_filter {
                q = q.filter(audit_logs::actor_id.eq(actor_id));
            }
            if let Some(entity_type) = &opts.entity_type_filter {
                q = q.filter(audit_logs::entity_type.eq(entity_type));
            }
            if let Some(entity_id) = opts.entity_filter {
                q = q.filter(audit_logs::entity_id.eq(entity_id));
            }

            if let Some(action_type) = &opts.action_type_filter {
                q = q.filter(audit_logs::action_type.eq(action_type));
            }

            q
        };

        let total_count = build_query().count().get_result::<i64>(conn)?;

        let logs = build_query()
            .order(audit_logs::timestamp.desc())
            .limit(page_query.per_page())
            .offset(page_query.offset())
            .select(AuditLogEntry::as_select())
            .load::<Self>(conn)?;

        Ok(Paginated::from_data(
            page_query,
            total_count,
            AuditLogPage { data: logs },
        ))
    }
}
