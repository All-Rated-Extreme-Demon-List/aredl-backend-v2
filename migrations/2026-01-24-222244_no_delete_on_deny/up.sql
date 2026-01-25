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
			
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;