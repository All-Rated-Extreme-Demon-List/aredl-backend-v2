ALTER TABLE audit_logs DROP COLUMN parent_entity_id;

DELETE FROM audit_logs
WHERE entity_type::text IN ('LevelCreator', 'LevelNote', 'LevelUpdate', 'PackTier', 'RoleUser');

ALTER TABLE audit_logs ALTER COLUMN entity_type TYPE TEXT USING entity_type::text;
DROP TYPE audit_entity_type;
CREATE TYPE audit_entity_type AS ENUM (
    'User', 'Level', 'Record', 'Pack', 'Role', 'Permission', 'Bounty',
    'LevelCustomCopy', 'SubmissionStatus', 'Clan', 'Shift'
);
ALTER TABLE audit_logs ALTER COLUMN entity_type TYPE audit_entity_type
    USING entity_type::audit_entity_type;
