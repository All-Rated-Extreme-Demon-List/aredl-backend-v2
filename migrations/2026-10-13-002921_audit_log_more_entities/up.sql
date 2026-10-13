ALTER TYPE audit_entity_type ADD VALUE 'LevelCreator';
ALTER TYPE audit_entity_type ADD VALUE 'LevelNote';
ALTER TYPE audit_entity_type ADD VALUE 'LevelUpdate';
ALTER TYPE audit_entity_type ADD VALUE 'PackTier';
ALTER TYPE audit_entity_type ADD VALUE 'RoleUser';

ALTER TABLE audit_logs ADD COLUMN parent_entity_id UUID;
