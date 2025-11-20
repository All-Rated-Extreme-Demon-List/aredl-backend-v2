
DROP VIEW aredl.submissions_with_priority;
DROP VIEW arepl.submissions_with_priority;

ALTER TABLE aredl.guideline_updates
    ALTER COLUMN created_at SET DEFAULT CLOCK_TIMESTAMP();

ALTER TABLE aredl.level_ldms
    ALTER COLUMN created_at SET DEFAULT CLOCK_TIMESTAMP();

ALTER TABLE arepl.level_ldms
    ALTER COLUMN created_at SET DEFAULT CLOCK_TIMESTAMP();

ALTER TABLE recurrent_shifts
	ALTER COLUMN created_at SET DEFAULT CLOCK_TIMESTAMP(),
	ALTER COLUMN updated_at SET DEFAULT CLOCK_TIMESTAMP();

ALTER TABLE shifts
	ALTER COLUMN created_at SET DEFAULT CLOCK_TIMESTAMP(),
	ALTER COLUMN updated_at SET DEFAULT CLOCK_TIMESTAMP();

ALTER TABLE aredl.submissions
	ALTER COLUMN created_at SET DEFAULT CLOCK_TIMESTAMP(),
	ALTER COLUMN updated_at SET DEFAULT CLOCK_TIMESTAMP();

ALTER TABLE arepl.submissions
	ALTER COLUMN created_at SET DEFAULT CLOCK_TIMESTAMP(),
	ALTER COLUMN updated_at SET DEFAULT CLOCK_TIMESTAMP();

ALTER TABLE aredl.submissions_enabled
	ALTER COLUMN created_at SET DEFAULT CLOCK_TIMESTAMP();

ALTER TABLE arepl.submissions_enabled
	ALTER COLUMN created_at SET DEFAULT CLOCK_TIMESTAMP();

ALTER TABLE aredl.submission_history
	ALTER COLUMN timestamp SET DEFAULT CLOCK_TIMESTAMP();

ALTER TABLE arepl.submission_history
	ALTER COLUMN timestamp SET DEFAULT CLOCK_TIMESTAMP();

CREATE OR REPLACE VIEW aredl.submissions_with_priority AS
SELECT 
    *,
    (EXTRACT(EPOCH FROM CLOCK_TIMESTAMP()) - EXTRACT(EPOCH FROM created_at))::BIGINT + 
    CASE WHEN priority = TRUE THEN 21600 ELSE 0 END AS priority_value
FROM aredl.submissions;

CREATE OR REPLACE VIEW arepl.submissions_with_priority AS
SELECT 
    *,
    (EXTRACT(EPOCH FROM CLOCK_TIMESTAMP()) - EXTRACT(EPOCH FROM created_at))::BIGINT + 
    CASE WHEN priority = TRUE THEN 21600 ELSE 0 END AS priority_value
FROM arepl.submissions;