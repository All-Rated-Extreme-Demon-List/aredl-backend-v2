CREATE OR REPLACE FUNCTION aredl.submission_log_history()
RETURNS TRIGGER AS
$$
DECLARE
    only_claim_toggle boolean;
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO aredl.submission_history (id, submission_id, status, user_notes, reviewer_id, reviewer_notes, private_reviewer_notes, locked, mobile, custom_copy_id, video_url, raw_url, mod_menu, priority, timestamp)
        VALUES (uuid_generate_v4(), NEW.id, NEW.status, NEW.user_notes, NEW.reviewer_id, NEW.reviewer_notes, NEW.private_reviewer_notes, NEW.locked, NEW.mobile, NEW.custom_copy_id, NEW.video_url, NEW.raw_url, NEW.mod_menu, NEW.priority, CLOCK_TIMESTAMP());
        RETURN NEW;
    END IF;

    IF TG_OP = 'UPDATE' THEN
        IF NEW IS NOT DISTINCT FROM OLD THEN
            RETURN NEW;
        END IF;

        only_claim_toggle = NEW.status <> OLD.status
            AND ((OLD.status = 'Pending' AND NEW.status = 'Claimed') OR (OLD.status = 'Claimed' AND NEW.status = 'Pending'))
            AND NEW.user_notes IS NOT DISTINCT FROM OLD.user_notes
            AND NEW.reviewer_notes IS NOT DISTINCT FROM OLD.reviewer_notes
            AND NEW.private_reviewer_notes IS NOT DISTINCT FROM OLD.private_reviewer_notes
            AND NEW.locked IS NOT DISTINCT FROM OLD.locked
            AND NEW.mobile IS NOT DISTINCT FROM OLD.mobile
            AND NEW.custom_copy_id IS NOT DISTINCT FROM OLD.custom_copy_id
            AND NEW.video_url IS NOT DISTINCT FROM OLD.video_url
            AND NEW.raw_url IS NOT DISTINCT FROM OLD.raw_url
            AND NEW.mod_menu IS NOT DISTINCT FROM OLD.mod_menu
            AND NEW.priority IS NOT DISTINCT FROM OLD.priority;

        IF only_claim_toggle THEN
            RETURN NEW;
        END IF;

        INSERT INTO aredl.submission_history (id, submission_id, status, user_notes, reviewer_id, reviewer_notes, private_reviewer_notes, locked, mobile, custom_copy_id, video_url, raw_url, mod_menu, priority, timestamp)
        VALUES (uuid_generate_v4(), NEW.id, NEW.status, NEW.user_notes, NEW.reviewer_id, NEW.reviewer_notes, NEW.private_reviewer_notes, NEW.locked, NEW.mobile, NEW.custom_copy_id, NEW.video_url, NEW.raw_url, NEW.mod_menu, NEW.priority, CLOCK_TIMESTAMP());

        RETURN NEW;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION arepl.submission_log_history()
RETURNS TRIGGER AS
$$
DECLARE
    only_claim_toggle boolean;
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO arepl.submission_history (id, submission_id, status, user_notes, reviewer_id, reviewer_notes, private_reviewer_notes, locked, mobile, custom_copy_id, video_url, raw_url, mod_menu, priority, completion_time, timestamp)
        VALUES (uuid_generate_v4(), NEW.id, NEW.status, NEW.user_notes, NEW.reviewer_id, NEW.reviewer_notes, NEW.private_reviewer_notes, NEW.locked, NEW.mobile, NEW.custom_copy_id, NEW.video_url, NEW.raw_url, NEW.mod_menu, NEW.priority, NEW.completion_time, CLOCK_TIMESTAMP());
        RETURN NEW;
    END IF;

    IF TG_OP = 'UPDATE' THEN
        IF NEW IS NOT DISTINCT FROM OLD THEN
            RETURN NEW;
        END IF;

        only_claim_toggle = NEW.status <> OLD.status
            AND ((OLD.status = 'Pending' AND NEW.status = 'Claimed') OR (OLD.status = 'Claimed' AND NEW.status = 'Pending'))
            AND NEW.user_notes IS NOT DISTINCT FROM OLD.user_notes
            AND NEW.reviewer_notes IS NOT DISTINCT FROM OLD.reviewer_notes
            AND NEW.private_reviewer_notes IS NOT DISTINCT FROM OLD.private_reviewer_notes
            AND NEW.locked IS NOT DISTINCT FROM OLD.locked
            AND NEW.mobile IS NOT DISTINCT FROM OLD.mobile
            AND NEW.custom_copy_id IS NOT DISTINCT FROM OLD.custom_copy_id
            AND NEW.video_url IS NOT DISTINCT FROM OLD.video_url
            AND NEW.raw_url IS NOT DISTINCT FROM OLD.raw_url
            AND NEW.mod_menu IS NOT DISTINCT FROM OLD.mod_menu
            AND NEW.priority IS NOT DISTINCT FROM OLD.priority
            AND NEW.completion_time IS NOT DISTINCT FROM OLD.completion_time;

        IF only_claim_toggle THEN
            RETURN NEW;
        END IF;

        INSERT INTO arepl.submission_history (id, submission_id, status, user_notes, reviewer_id, reviewer_notes, private_reviewer_notes, locked, mobile, custom_copy_id, video_url, raw_url, mod_menu, priority, completion_time, timestamp)
        VALUES (uuid_generate_v4(), NEW.id, NEW.status, NEW.user_notes, NEW.reviewer_id, NEW.reviewer_notes, NEW.private_reviewer_notes, NEW.locked, NEW.mobile, NEW.custom_copy_id, NEW.video_url, NEW.raw_url, NEW.mod_menu, NEW.priority, NEW.completion_time, CLOCK_TIMESTAMP());

        RETURN NEW;
    END IF;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION aredl.submission_updated_at()
RETURNS TRIGGER AS
$$
BEGIN
    IF NEW IS DISTINCT FROM OLD THEN
        NEW.updated_at = CLOCK_TIMESTAMP();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION arepl.submission_updated_at()
RETURNS TRIGGER AS
$$
BEGIN
    IF NEW IS DISTINCT FROM OLD THEN
        NEW.updated_at = CLOCK_TIMESTAMP();
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP FUNCTION aredl.submission_is_only_claim_toggle(aredl.submissions, aredl.submissions);
DROP FUNCTION arepl.submission_is_only_claim_toggle(arepl.submissions, arepl.submissions);
