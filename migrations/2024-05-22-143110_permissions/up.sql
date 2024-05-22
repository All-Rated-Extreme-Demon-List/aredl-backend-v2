CREATE TABLE roles (
    id SERIAL,
    privilege_level int NOT NULL,
    role_desc VARCHAR NOT NULL,
    PRIMARY KEY(id)
);

CREATE TABLE user_roles (
    role_id int NOT NULL REFERENCES roles(id),
    user_id uuid NOT NULL REFERENCES users(id),
    PRIMARY KEY(role_id, user_id)
);

CREATE TABLE permissions (
    privilege_level int NOT NULL,
    permission VARCHAR NOT NULL,
    PRIMARY KEY(permission)
);