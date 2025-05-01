CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id uuid DEFAULT uuid_generate_v4(),
    username VARCHAR NOT NULL DEFAULT substring(md5(random()::text), 0, 10),
    json_id BIGINT,
    global_name VARCHAR NOT NULL,
    discord_id VARCHAR,
    placeholder BOOLEAN NOT NULL,
    description TEXT,
    country INTEGER,
    last_country_update TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01 00:00:00+00',
    ban_level INTEGER NOT NULL DEFAULT 0,
    discord_avatar VARCHAR,
    discord_banner VARCHAR,
    discord_accent_color int,
    access_valid_after TIMESTAMPTZ NOT NULL DEFAULT '1970-01-01 00:00:00+00',
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(id),
    UNIQUE(username),
    UNIQUE(discord_id)
);