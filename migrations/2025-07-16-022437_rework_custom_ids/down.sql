-- This file should undo anything in `up.sql`
ALTER TABLE aredl.level_ldms DROP COLUMN "id_type";
ALTER TABLE aredl.level_ldms DROP COLUMN "status";
ALTER TABLE aredl.level_ldms ADD COLUMN 
    is_allowed BOOLEAN NOT NULL DEFAULT true;

DROP TYPE aredl.custom_id_type;
DROP TYPE aredl.custom_id_status;

ALTER TABLE arepl.level_ldms DROP COLUMN "id_type";
ALTER TABLE arepl.level_ldms DROP COLUMN "status";
ALTER TABLE arepl.level_ldms ADD COLUMN 
    is_allowed BOOLEAN NOT NULL DEFAULT true;

DROP TYPE arepl.custom_id_type;
DROP TYPE arepl.custom_id_status;