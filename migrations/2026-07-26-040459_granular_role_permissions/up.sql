ALTER TABLE roles
    ADD COLUMN inherits_from_role_id INTEGER NULL REFERENCES roles(id) ON DELETE SET NULL,
    ADD CONSTRAINT roles_cannot_inherit_self CHECK (
        inherits_from_role_id IS NULL OR inherits_from_role_id <> id
    );

CREATE TABLE role_permissions (
    role_id INTEGER NOT NULL REFERENCES roles(id) ON DELETE CASCADE,
    permission VARCHAR NOT NULL REFERENCES permissions(permission) ON DELETE CASCADE,
    PRIMARY KEY(role_id, permission)
);

INSERT INTO role_permissions (role_id, permission)
SELECT roles.id, permissions.permission
FROM roles
INNER JOIN permissions ON permissions.privilege_level <= roles.privilege_level
ON CONFLICT DO NOTHING;

CREATE OR REPLACE FUNCTION prevent_role_inheritance_cycle()
RETURNS TRIGGER AS $$
DECLARE
    has_cycle BOOLEAN;
BEGIN
    IF NEW.inherits_from_role_id IS NULL THEN
        RETURN NEW;
    END IF;

    WITH RECURSIVE inherited_roles(id) AS (
        SELECT NEW.inherits_from_role_id
        UNION
        SELECT roles.inherits_from_role_id
        FROM roles
        INNER JOIN inherited_roles ON roles.id = inherited_roles.id
        WHERE roles.inherits_from_role_id IS NOT NULL
    )
    SELECT EXISTS (
        SELECT 1
        FROM inherited_roles
        WHERE id = NEW.id
    )
    INTO has_cycle;

    IF has_cycle THEN
        RAISE EXCEPTION 'Role inheritance cycle detected for role %', NEW.id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER prevent_role_inheritance_cycle
    BEFORE INSERT OR UPDATE OF inherits_from_role_id ON roles
    FOR EACH ROW
    EXECUTE FUNCTION prevent_role_inheritance_cycle();

CREATE VIEW role_permissions_full AS
WITH RECURSIVE inherited_roles(role_id, inherited_role_id) AS (
    SELECT roles.id, roles.id
    FROM roles

    UNION

    SELECT inherited_roles.role_id, roles.inherits_from_role_id
    FROM inherited_roles
    INNER JOIN roles ON roles.id = inherited_roles.inherited_role_id
    WHERE roles.inherits_from_role_id IS NOT NULL
)
SELECT DISTINCT inherited_roles.role_id, role_permissions.permission
FROM inherited_roles
INNER JOIN role_permissions ON role_permissions.role_id = inherited_roles.inherited_role_id;

ALTER TABLE permissions
    DROP COLUMN IF EXISTS privilege_level;
