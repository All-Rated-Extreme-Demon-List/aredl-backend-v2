ALTER TABLE submission_history 
RENAME COLUMN rejection_reason TO reviewer_notes;

ALTER TABLE submission_history
ADD COLUMN IF NOT EXISTS user_notes TEXT;

ALTER TABLE submission_history
ADD COLUMN IF NOT EXISTS reviewer_id UUID;

ALTER TABLE aredl_submissions
ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP NOT NULL DEFAULT NOW();