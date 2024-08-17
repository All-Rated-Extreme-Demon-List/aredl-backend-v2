CREATE TABLE users (
    id uuid DEFAULT uuid_generate_v4(),
    username VARCHAR NOT NULL DEFAULT substring(md5(random()::text), 0, 10),
    global_name VARCHAR NOT NULL,
    discord_id VARCHAR,
    placeholder BOOLEAN NOT NULL,
    country INTEGER,
    ban_level INTEGER NOT NULL DEFAULT 0,
    discord_avatar VARCHAR,
    discord_banner VARCHAR,
    discord_accent_color int,
    PRIMARY KEY(id),
    UNIQUE(username),
    UNIQUE(discord_id)
);