CREATE OR REPLACE FUNCTION aredl.submission_sync_record()
RETURNS TRIGGER AS
$$
BEGIN
    IF NEW.status = 'Accepted' THEN
        INSERT INTO aredl.records AS r (
            level_id,
            submitted_by,
            mobile,
            video_url,
            submission_id
        )
        VALUES (
            NEW.level_id,
            NEW.submitted_by,
            NEW.mobile,
            NEW.video_url,
            NEW.id
        )
        ON CONFLICT (level_id, submitted_by)
        DO UPDATE SET
            mobile = EXCLUDED.mobile,
            video_url = EXCLUDED.video_url;

    ELSIF NEW.status = 'Denied'
      AND (OLD.status IS DISTINCT FROM NEW.status)
    THEN
        DELETE FROM aredl.records
        WHERE submission_id = NEW.id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;


CREATE OR REPLACE FUNCTION arepl.submission_sync_record()
RETURNS TRIGGER AS
$$
BEGIN
    IF NEW.status = 'Accepted' THEN
        INSERT INTO arepl.records AS r (
            level_id,
            submitted_by,
            mobile,
            video_url,
			completion_time,
            submission_id
        )
        VALUES (
            NEW.level_id,
            NEW.submitted_by,
            NEW.mobile,
            NEW.video_url,
			NEW.completion_time,
            NEW.id
        )
        ON CONFLICT (level_id, submitted_by)
        DO UPDATE SET
            mobile = EXCLUDED.mobile,
            video_url = EXCLUDED.video_url,
			completion_time = EXCLUDED.completion_time;

    ELSIF NEW.status = 'Denied'
      AND (OLD.status IS DISTINCT FROM NEW.status)
    THEN
        DELETE FROM arepl.records
        WHERE submission_id = NEW.id;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;