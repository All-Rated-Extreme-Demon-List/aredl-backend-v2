-- This file should undo anything in `up.sql`

DROP INDEX audit_time_idx;
DROP INDEX audit_actor_idx;
DROP INDEX audit_entity_idx;

DROP TABLE audit_logs;

DROP TYPE audit_entity_type;
DROP TYPE audit_action;