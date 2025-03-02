DROP TRIGGER IF EXISTS update_aredl_record_placement_on_update ON aredl_records;

DROP TRIGGER IF EXISTS update_aredl_record_placement_on_insert ON aredl_records;

DROP FUNCTION IF EXISTS update_aredl_record_placement;

DROP TRIGGER IF EXISTS update_aredl_record_time ON aredl_records;

DROP FUNCTION IF EXISTS update_aredl_record_time;

DROP TABLE IF EXISTS aredl_submissions;

DROP TABLE IF EXISTS aredl_records;