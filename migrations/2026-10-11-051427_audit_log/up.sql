CREATE TYPE audit_action AS ENUM ('Create', 'Update', 'Delete');
CREATE TYPE audit_entity_type AS ENUM (
    'User',
    'Level',
    'Record',
    'Pack',
    'Role',
    'Permission',
    'Bounty',
    'LevelCustomCopy',
    'SubmissionStatus',
    'Clan',
    'Shift'
);

CREATE TABLE audit_logs (
    id uuid PRIMARY KEY DEFAULT uuid_generate_v4(),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    list_id SMALLINT REFERENCES lists(id),
    actor_id uuid, -- null = automatic/system action
    action_type audit_action NOT NULL,
    entity_id uuid NOT NULL,
    entity_type audit_entity_type NOT NULL,
    entity_display TEXT NOT NULL,
    parent_display TEXT,
    before JSONB,
    after JSONB
);

CREATE INDEX audit_entity_idx ON audit_logs (entity_type, entity_id, list_id, timestamp DESC);
CREATE INDEX audit_actor_idx  ON audit_logs (actor_id, timestamp DESC);
CREATE INDEX audit_time_idx   ON audit_logs USING brin (timestamp);