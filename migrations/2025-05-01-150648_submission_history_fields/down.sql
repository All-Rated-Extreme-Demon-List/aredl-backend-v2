ALTER TABLE submission_history
	RENAME COLUMN reviewer_notes TO rejection_reason;

ALTER TABLE submission_history
	DROP COLUMN IF EXISTS user_notes;

ALTER TABLE submission_history
	DROP COLUMN IF EXISTS reviewer_id;

ALTER TABLE aredl_submissions
	DROP COLUMN IF EXISTS updated_at;