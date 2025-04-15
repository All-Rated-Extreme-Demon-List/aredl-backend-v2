ALTER TABLE aredl_submissions 
    DROP COLUMN status

DROP TYPE submission_status;

ALTER TABLE aredl_submissions
    CREATE COLUMN is_rejected boolean DEFAULT false NOT NULL;


