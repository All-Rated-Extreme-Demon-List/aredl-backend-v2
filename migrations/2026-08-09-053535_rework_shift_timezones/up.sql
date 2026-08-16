-- Your SQL goes here
ALTER TABLE recurrent_shifts ADD COLUMN timezone VARCHAR(50) NOT NULL DEFAULT 'UTC';