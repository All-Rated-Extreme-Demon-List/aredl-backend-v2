DROP VIEW IF EXISTS aredl_submissions_with_priority;

ALTER TABLE aredl_submissions
	RENAME COLUMN rejection_reason TO reviewer_notes;

ALTER TABLE aredl_submissions
	RENAME COLUMN additional_notes TO user_notes;

ALTER TABLE aredl_submissions
	DROP COLUMN IF EXISTS is_update;


ALTER TABLE aredl_records
	ADD COLUMN IF NOT EXISTS reviewer_notes VARCHAR;

ALTER TABLE aredl_records
	ADD COLUMN IF NOT EXISTS mod_menu VARCHAR;

ALTER TABLE aredl_records
	ADD COLUMN IF NOT EXISTS user_notes VARCHAR;

CREATE OR REPLACE VIEW aredl_submissions_with_priority AS
SELECT 
    *,
    -- epoch is # of seconds passed since 1970
    (EXTRACT(EPOCH FROM NOW()) - EXTRACT(EPOCH FROM created_at))::BIGINT + 
    -- 21600 is # of seconds in 6
    CASE WHEN priority = TRUE THEN 21600 ELSE 0 END AS priority_value
FROM aredl_submissions;
