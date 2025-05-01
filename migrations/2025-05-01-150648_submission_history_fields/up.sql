DROP VIEW IF EXISTS aredl_submissions_with_priority;

ALTER TABLE submission_history 
RENAME COLUMN rejection_reason TO reviewer_notes;

ALTER TABLE submission_history
ADD COLUMN IF NOT EXISTS user_notes TEXT;

ALTER TABLE submission_history
ADD COLUMN IF NOT EXISTS reviewer_id UUID;

ALTER TABLE aredl_submissions
ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW();

CREATE OR REPLACE VIEW aredl_submissions_with_priority AS
SELECT 
    *,
    -- epoch is # of seconds passed since 1970
    (EXTRACT(EPOCH FROM NOW()) - EXTRACT(EPOCH FROM created_at))::BIGINT + 
    -- 21600 is # of seconds in 6
    CASE WHEN priority = TRUE THEN 21600 ELSE 0 END AS priority_value
FROM aredl_submissions;
