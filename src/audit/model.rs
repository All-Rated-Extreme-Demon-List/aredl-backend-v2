use crate::{
    app_data::db::DbConnection,
    error_handler::ApiError,
    page_helper::{PageQuery, Paginated},
    schema::audit_logs,
};
use chrono::{DateTime, Utc};
use diesel::{
    pg::Pg, ExpressionMethods as _, QueryDsl as _, Queryable, RunQueryDsl as _, Selectable,
    SelectableHelper as _,
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
    pub diff: Value,
    pub timestamp: DateTime<Utc>,
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
