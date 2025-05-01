CREATE TABLE aredl_records (
    id uuid DEFAULT uuid_generate_v4(),
    level_id uuid NOT NULL REFERENCES aredl_levels(id) ON DELETE CASCADE ON UPDATE CASCADE,
    submitted_by uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE ON UPDATE CASCADE,
    mobile boolean NOT NULL DEFAULT false,
    ldm_id int,
    video_url VARCHAR NOT NULL,
    raw_url VARCHAR,
    placement_order int DEFAULT 0 NOT NULL,
    reviewer_id uuid REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
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
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP NOT NULL,
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

CREATE FUNCTION update_aredl_record_placement()
RETURNS TRIGGER AS $$
  BEGIN
    UPDATE aredl_records
    SET placement_order = sub.row_num - 1
    FROM (
        SELECT id, ROW_NUMBER() OVER (PARTITION BY level_id ORDER BY created_at) AS row_num
        FROM aredl_records
        WHERE EXISTS (
            SELECT 1 FROM new_table as n WHERE n.level_id = aredl_records.level_id
        )
    ) AS sub
    WHERE aredl_records.id = sub.id;
    RETURN null;
  END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_aredl_record_placement_on_update
AFTER UPDATE ON aredl_records
REFERENCING NEW TABLE as new_table
FOR EACH STATEMENT
WHEN (pg_trigger_depth() < 1)
EXECUTE FUNCTION update_aredl_record_placement();

CREATE TRIGGER update_aredl_record_placement_on_insert
AFTER INSERT ON aredl_records
REFERENCING NEW TABLE as new_table
FOR EACH STATEMENT
WHEN (pg_trigger_depth() < 1)
EXECUTE FUNCTION update_aredl_record_placement();