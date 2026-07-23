CREATE TYPE aredl.level_notes_type AS ENUM('ReviewerNotes', 'PublicNotes', 'Other');
CREATE TYPE arepl.level_notes_type AS ENUM('ReviewerNotes', 'PublicNotes', 'Other');

CREATE TYPE aredl.level_update_type AS ENUM('Buff', 'Nerf', 'Balance', 'BugFix', 'Other');
CREATE TYPE arepl.level_update_type AS ENUM('Buff', 'Nerf', 'Balance', 'BugFix', 'Other');

CREATE TABLE aredl.level_notes (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    level_id UUID NOT NULL REFERENCES aredl.levels(id) ON DELETE CASCADE ON UPDATE CASCADE,
    note TEXT NOT NULL,
	note_type aredl.level_notes_type NOT NULL,
    timestamp TIMESTAMPTZ,
    added_by UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL ON UPDATE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CLOCK_TIMESTAMP()
);

CREATE TABLE arepl.level_notes (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    level_id UUID NOT NULL REFERENCES arepl.levels(id) ON DELETE CASCADE ON UPDATE CASCADE,
    note TEXT NOT NULL,
	note_type arepl.level_notes_type NOT NULL,
    timestamp TIMESTAMPTZ,
    added_by UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL ON UPDATE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CLOCK_TIMESTAMP()
);

CREATE TABLE aredl.level_updates (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    level_id UUID NOT NULL REFERENCES aredl.levels(id) ON DELETE CASCADE ON UPDATE CASCADE,
    changelog TEXT,
    update_type aredl.level_update_type NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CLOCK_TIMESTAMP()
);

CREATE TABLE arepl.level_updates (
    id UUID NOT NULL DEFAULT gen_random_uuid() PRIMARY KEY,
    level_id UUID NOT NULL REFERENCES arepl.levels(id) ON DELETE CASCADE ON UPDATE CASCADE,
    changelog TEXT,
    update_type arepl.level_update_type NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CLOCK_TIMESTAMP()
);
