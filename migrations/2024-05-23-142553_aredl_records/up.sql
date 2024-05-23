CREATE TABLE aredl_records (
    id uuid DEFAULT uuid_generate_v4(),
    level_id uuid NOT NULL REFERENCES aredl_levels(id) ON DELETE CASCADE ON UPDATE CASCADE,
    submitted_by uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    mobile boolean NOT NULL DEFAULT false,
    ldm_id int,
    video_url VARCHAR NOT NULL,
    raw_url VARCHAR,
    reviewer_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY(id),
    UNIQUE(level_id, submitted_by)
);

CREATE TABLE aredl_submissions (
    id uuid DEFAULT uuid_generate_v4(),
    level_id uuid NOT NULL REFERENCES aredl_levels(id) ON DELETE CASCADE ON UPDATE CASCADE,
    submitted_by uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    mobile boolean NOT NULL DEFAULT false,
    ldm_id int,
    video_url VARCHAR NOT NULL,
    raw_url VARCHAR,
    reviewer_id uuid REFERENCES users(id) ON DELETE SET NULL,
    priority boolean DEFAULT false NOT NULL,
    is_update boolean DEFAULT false NOT NULL,
    is_rejected boolean DEFAULT false NOT NULL,
    rejection_reason VARCHAR,
    additional_notes VARCHAR,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    PRIMARY KEY(id),
    UNIQUE(level_id, submitted_by)
);

CREATE FUNCTION update_aredl_record_time()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_aredl_record_time
BEFORE UPDATE ON aredl_records
FOR EACH ROW
EXECUTE FUNCTION update_aredl_record_time();