DROP VIEW role_permissions_full;
DROP TRIGGER prevent_role_inheritance_cycle ON roles;
ALTER TABLE roles
    DROP CONSTRAINT roles_inherits_from_role_id_fkey,
    DROP CONSTRAINT roles_cannot_inherit_self;
ALTER TABLE user_roles DROP CONSTRAINT user_roles_role_id_fkey;
ALTER TABLE role_permissions DROP CONSTRAINT role_permissions_role_id_fkey;

CREATE TEMP TABLE role_int_map (
    old_id UUID PRIMARY KEY,
    new_id INTEGER NOT NULL UNIQUE
) ON COMMIT DROP;
INSERT INTO role_int_map (old_id, new_id)
SELECT id, (ROW_NUMBER() OVER (ORDER BY id))::INTEGER FROM roles;

CREATE FUNCTION pg_temp.old_role_id(UUID) RETURNS INTEGER
LANGUAGE SQL STABLE STRICT AS $$
    SELECT new_id FROM role_int_map WHERE old_id = $1;
$$;

ALTER TABLE roles ALTER COLUMN id DROP DEFAULT;
ALTER TABLE roles
    ALTER COLUMN id TYPE INTEGER USING pg_temp.old_role_id(id),
    ALTER COLUMN inherits_from_role_id TYPE INTEGER
        USING pg_temp.old_role_id(inherits_from_role_id);
ALTER TABLE user_roles ALTER COLUMN role_id TYPE INTEGER
    USING pg_temp.old_role_id(role_id);
ALTER TABLE role_permissions ALTER COLUMN role_id TYPE INTEGER
    USING pg_temp.old_role_id(role_id);

DROP FUNCTION pg_temp.old_role_id(UUID);
CREATE SEQUENCE roles_id_seq OWNED BY roles.id;
SELECT setval('roles_id_seq', COALESCE(MAX(id), 1), MAX(id) IS NOT NULL) FROM roles;
ALTER TABLE roles
    ALTER COLUMN id SET DEFAULT nextval('roles_id_seq'),
    ADD CONSTRAINT roles_inherits_from_role_id_fkey
        FOREIGN KEY (inherits_from_role_id) REFERENCES roles(id) ON DELETE SET NULL,
    ADD CONSTRAINT roles_cannot_inherit_self CHECK (
        inherits_from_role_id IS NULL OR inherits_from_role_id <> id
    );
CREATE TRIGGER prevent_role_inheritance_cycle
    BEFORE INSERT OR UPDATE OF inherits_from_role_id ON roles
    FOR EACH ROW EXECUTE FUNCTION prevent_role_inheritance_cycle();
ALTER TABLE user_roles ADD CONSTRAINT user_roles_role_id_fkey
    FOREIGN KEY (role_id) REFERENCES roles(id);
ALTER TABLE role_permissions ADD CONSTRAINT role_permissions_role_id_fkey
    FOREIGN KEY (role_id) REFERENCES roles(id) ON DELETE CASCADE;

CREATE VIEW role_permissions_full AS
WITH RECURSIVE inherited_roles(role_id, inherited_role_id) AS (
    SELECT roles.id, roles.id FROM roles
    UNION
    SELECT inherited_roles.role_id, roles.inherits_from_role_id
    FROM inherited_roles
    INNER JOIN roles ON roles.id = inherited_roles.inherited_role_id
    WHERE roles.inherits_from_role_id IS NOT NULL
)
SELECT DISTINCT inherited_roles.role_id, role_permissions.permission
FROM inherited_roles
INNER JOIN role_permissions ON role_permissions.role_id = inherited_roles.inherited_role_id;
