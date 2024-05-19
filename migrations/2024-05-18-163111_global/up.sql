CREATE TABLE users (
    id uuid DEFAULT uuid_generate_v4(),
    user_name VARCHAR NOT NULL DEFAULT substring(md5(random()::text), 0, 10),
    global_name VARCHAR NOT NULL,
    placeholder BOOLEAN NOT NULL,
    PRIMARY KEY(id),
    UNIQUE(user_name)
);