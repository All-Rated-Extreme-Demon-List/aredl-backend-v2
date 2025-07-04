CREATE TABLE aredl.guideline_updates (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    moderator UUID NOT NULL REFERENCES users(id),
    text VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
