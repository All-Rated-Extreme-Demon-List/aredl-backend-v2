DROP VIEW IF EXISTS role_permissions_full;

DROP TRIGGER IF EXISTS prevent_role_inheritance_cycle ON roles;

DROP FUNCTION IF EXISTS prevent_role_inheritance_cycle();

DROP TABLE IF EXISTS role_permissions;

DROP INDEX IF EXISTS user_roles_user_id_idx;

ALTER TABLE roles
    DROP CONSTRAINT IF EXISTS roles_cannot_inherit_self,
    DROP COLUMN IF EXISTS inherits_from_role_id;

ALTER TABLE permissions
    ADD COLUMN privilege_level INTEGER NOT NULL DEFAULT 0;
