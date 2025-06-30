CREATE TABLE aredl.level_ldms (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    level_id UUID NOT NULL REFERENCES aredl.levels(id) ON DELETE CASCADE ON UPDATE CASCADE,
    ldm_id INT NOT NULL CHECK (ldm_id > 0),
    is_allowed BOOLEAN NOT NULL DEFAULT true,
    added_by UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL ON UPDATE CASCADE,
    description VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE arepl.level_ldms (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    level_id UUID NOT NULL REFERENCES arepl.levels(id) ON DELETE CASCADE ON UPDATE CASCADE,
    ldm_id INT NOT NULL CHECK (ldm_id > 0),
    is_allowed BOOLEAN NOT NULL DEFAULT true,
    added_by UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL ON UPDATE CASCADE,
    description VARCHAR,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);