CREATE OR REPLACE FUNCTION aredl.submission_updated_at()
RETURNS TRIGGER AS
$$
BEGIN
    IF NEW IS DISTINCT FROM OLD THEN
        IF aredl.submission_is_only_claim_toggle(OLD, NEW) THEN
            NEW.updated_at = OLD.updated_at;
        ELSE
            NEW.updated_at = CLOCK_TIMESTAMP();
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION arepl.submission_updated_at()
RETURNS TRIGGER AS
$$
BEGIN
    IF NEW IS DISTINCT FROM OLD THEN
        IF arepl.submission_is_only_claim_toggle(OLD, NEW) THEN
            NEW.updated_at = OLD.updated_at;
        ELSE
            NEW.updated_at = CLOCK_TIMESTAMP();
        END IF;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

ALTER TABLE aredl.submissions DROP COLUMN priority_at;
ALTER TABLE arepl.submissions DROP COLUMN priority_at;
