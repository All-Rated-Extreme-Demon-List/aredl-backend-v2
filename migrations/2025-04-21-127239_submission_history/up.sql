-- Create the enum type if it doesn't exist
DO $$ BEGIN
    CREATE TYPE submission_status AS ENUM ('Pending', 'Claimed', 'UnderConsideration', 'Denied');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create the submission_history table
CREATE TABLE submission_history (
    id UUID PRIMARY KEY,
    submission_id UUID,
    record_id UUID,
    rejection_reason TEXT,
    status submission_status NOT NULL,
    timestamp TIMESTAMP NOT NULL DEFAULT NOW()
);
