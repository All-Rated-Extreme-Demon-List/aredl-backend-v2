-- This file should undo anything in `up.sql`
ALTER TABLE aredl.records DROP COLUMN IF EXISTS hide_video;
ALTER TABLE arepl.records DROP COLUMN IF EXISTS hide_video;