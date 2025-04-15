CREATE TYPE submission_status AS ENUM 
    ('Pending', 'Claimed', 'UnderConsideration', 'Denied', 'Accepted');

ALTER TABLE aredl_submissions 
    ADD COLUMN status submission_status NOT NULL DEFAULT 'Pending';

ALTER TABLE aredl_submissions 
    DROP COLUMN is_rejected;
