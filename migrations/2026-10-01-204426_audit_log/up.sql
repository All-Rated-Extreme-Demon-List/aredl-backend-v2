-- Your SQL goes here

CREATE TYPE audit_action AS ENUM ('Create', 'Update', 'Delete');
CREATE TYPE audit_entity_type AS ENUM (
    'User',
    'Level',
    'Record',
    'Pack',
    'Role',
    'Permission',
    'Bounty',
    'CustomCopy',
    'SubmissionStatus',
    'Clan',
    'Shift'
);

CREATE TABLE audit_logs (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    actor_id uuid REFERENCES users(id) ON DELETE SET NULL, -- null = automatic/system action or deleted user
    action_type audit_action NOT NULL,
    entity_id uuid NOT NULL,
    entity_type audit_entity_type NOT NULL,
    diff JSONB NOT NULL
);

CREATE INDEX audit_entity_idx ON audit_logs (entity_type, entity_id, timestamp DESC);
CREATE INDEX audit_actor_idx  ON audit_logs (actor_id, timestamp DESC);
CREATE INDEX audit_time_idx   ON audit_logs USING brin (timestamp);