CREATE OR REPLACE FUNCTION aredl.submission_is_only_claim_toggle(
    old_submission aredl.submissions,
    new_submission aredl.submissions
)
RETURNS boolean AS
$$
BEGIN
    RETURN new_submission.status <> old_submission.status
        AND (
            (old_submission.status = 'Pending' AND new_submission.status = 'Claimed')
            OR (old_submission.status = 'Claimed' AND new_submission.status = 'Pending')
        )
        AND (to_jsonb(new_submission) - 'updated_at' - 'status' - 'reviewer_id')
            IS NOT DISTINCT FROM
            (to_jsonb(old_submission) - 'updated_at' - 'status' - 'reviewer_id');
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION arepl.submission_is_only_claim_toggle(
    old_submission arepl.submissions,
    new_submission arepl.submissions
)
RETURNS boolean AS
$$
BEGIN
    RETURN new_submission.status <> old_submission.status
        AND (
            (old_submission.status = 'Pending' AND new_submission.status = 'Claimed')
            OR (old_submission.status = 'Claimed' AND new_submission.status = 'Pending')
        )
        AND (to_jsonb(new_submission) - 'updated_at' - 'status' - 'reviewer_id')
            IS NOT DISTINCT FROM
            (to_jsonb(old_submission) - 'updated_at' - 'status' - 'reviewer_id');
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION aredl.submission_log_history()
RETURNS TRIGGER AS
$$
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

        IF aredl.submission_is_only_claim_toggle(OLD, NEW) THEN
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

        IF arepl.submission_is_only_claim_toggle(OLD, NEW) THEN
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
