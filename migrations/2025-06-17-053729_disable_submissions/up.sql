CREATE TABLE IF NOT EXISTS aredl.submissions_enabled (
    id UUID NOT NULL DEFAULT gen_random_uuid(),
    enabled BOOL NOT NULL,
    moderator UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL ON UPDATE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY(id)
);

CREATE TABLE IF NOT EXISTS arepl.submissions_enabled (
    LIKE aredl.submissions_enabled INCLUDING ALL
);