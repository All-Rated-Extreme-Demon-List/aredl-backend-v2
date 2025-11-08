CREATE TYPE aredl.custom_id_type AS ENUM ('Bugfix', 'GlobedCopy', 'Ldm', 'Other');
CREATE TYPE aredl.custom_id_status AS ENUM ('Published', 'Allowed', 'Banned');

ALTER TABLE aredl.level_ldms DROP COLUMN is_allowed;
ALTER TABLE aredl.level_ldms ADD COLUMN 
    "id_type" aredl.custom_id_type NOT NULL;
ALTER TABLE aredl.level_ldms ADD COLUMN 
    "status" aredl.custom_id_status NOT NULL;

CREATE TYPE arepl.custom_id_type AS ENUM ('Bugfix', 'GlobedCopy', 'Ldm', 'Other');
CREATE TYPE arepl.custom_id_status AS ENUM ('Published', 'Allowed', 'Banned');

ALTER TABLE arepl.level_ldms DROP COLUMN is_allowed;
ALTER TABLE arepl.level_ldms ADD COLUMN 
    "id_type" arepl.custom_id_type NOT NULL;
ALTER TABLE arepl.level_ldms ADD COLUMN 
    "status" arepl.custom_id_status NOT NULL;