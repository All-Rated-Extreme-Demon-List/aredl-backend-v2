ALTER TABLE aredl.submissions
    ADD COLUMN priority_at TIMESTAMPTZ NOT NULL DEFAULT CLOCK_TIMESTAMP();

ALTER TABLE arepl.submissions
    ADD COLUMN priority_at TIMESTAMPTZ NOT NULL DEFAULT CLOCK_TIMESTAMP();

ALTER TABLE aredl.submissions DISABLE TRIGGER submission_updated_at;
ALTER TABLE aredl.submissions DISABLE TRIGGER submission_log_history_upd;
UPDATE aredl.submissions SET priority_at = updated_at;
ALTER TABLE aredl.submissions ENABLE TRIGGER submission_log_history_upd;
ALTER TABLE aredl.submissions ENABLE TRIGGER submission_updated_at;

ALTER TABLE arepl.submissions DISABLE TRIGGER submission_updated_at;
ALTER TABLE arepl.submissions DISABLE TRIGGER submission_log_history_upd;
UPDATE arepl.submissions SET priority_at = updated_at;
ALTER TABLE arepl.submissions ENABLE TRIGGER submission_log_history_upd;
ALTER TABLE arepl.submissions ENABLE TRIGGER submission_updated_at;

CREATE OR REPLACE FUNCTION aredl.submission_updated_at()
RETURNS TRIGGER AS
$$
DECLARE
    update_timestamp timestamptz;
BEGIN
    IF NEW IS DISTINCT FROM OLD THEN
        update_timestamp = CLOCK_TIMESTAMP();

        IF OLD.priority = FALSE AND NEW.priority = TRUE THEN
            NEW.priority_at = update_timestamp;
        END IF;

        NEW.updated_at = update_timestamp;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION arepl.submission_updated_at()
RETURNS TRIGGER AS
$$
DECLARE
    update_timestamp timestamptz;
BEGIN
    IF NEW IS DISTINCT FROM OLD THEN
        update_timestamp = CLOCK_TIMESTAMP();

        IF OLD.priority = FALSE AND NEW.priority = TRUE THEN
            NEW.priority_at = update_timestamp;
        END IF;

        NEW.updated_at = update_timestamp;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
