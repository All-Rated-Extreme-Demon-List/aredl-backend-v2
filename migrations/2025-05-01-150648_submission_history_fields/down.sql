ALTER TABLE submission_history
	RENAME COLUMN reviewer_notes TO rejection_reason;

ALTER TABLE submission_history
	DROP COLUMN IF EXISTS user_notes;

ALTER TABLE submission_history
	DROP COLUMN IF EXISTS reviewer_id;

ALTER TABLE aredl_submissions
	DROP COLUMN IF EXISTS updated_at;

DROP VIEW IF EXISTS aredl_submissions_with_priority;

CREATE OR REPLACE VIEW aredl_submissions_with_priority AS
SELECT 
    *,
    -- epoch is # of seconds passed since 1970
    (EXTRACT(EPOCH FROM NOW()) - EXTRACT(EPOCH FROM created_at))::BIGINT + 
    -- 21600 is # of seconds in 6
    CASE WHEN priority = TRUE THEN 21600 ELSE 0 END AS priority_value
FROM aredl_submissions;
