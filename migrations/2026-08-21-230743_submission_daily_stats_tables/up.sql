CREATE TABLE aredl.submission_daily_total_stats (
    day date PRIMARY KEY,
    submitted bigint NOT NULL DEFAULT 0 CHECK (submitted >= 0),
    accepted bigint NOT NULL DEFAULT 0 CHECK (accepted >= 0),
    denied bigint NOT NULL DEFAULT 0 CHECK (denied >= 0),
    under_consideration bigint NOT NULL DEFAULT 0 CHECK (under_consideration >= 0),
    reviewed bigint NOT NULL DEFAULT 0 CHECK (reviewed >= 0)
);

CREATE TABLE aredl.submission_daily_reviewer_stats (
    day date NOT NULL,
    reviewer_id uuid NOT NULL,
    accepted bigint NOT NULL DEFAULT 0 CHECK (accepted >= 0),
    denied bigint NOT NULL DEFAULT 0 CHECK (denied >= 0),
    under_consideration bigint NOT NULL DEFAULT 0 CHECK (under_consideration >= 0),
    reviewed bigint NOT NULL DEFAULT 0 CHECK (reviewed >= 0),
    PRIMARY KEY (day, reviewer_id)
);

CREATE INDEX aredl_submission_daily_reviewer_stats_reviewer_day_idx
    ON aredl.submission_daily_reviewer_stats (reviewer_id, day DESC);

CREATE TABLE aredl.submission_daily_level_stats (
    day date NOT NULL,
    level_id uuid NOT NULL,
    submitted bigint NOT NULL DEFAULT 0 CHECK (submitted >= 0),
    accepted bigint NOT NULL DEFAULT 0 CHECK (accepted >= 0),
    denied bigint NOT NULL DEFAULT 0 CHECK (denied >= 0),
    under_consideration bigint NOT NULL DEFAULT 0 CHECK (under_consideration >= 0),
    reviewed bigint NOT NULL DEFAULT 0 CHECK (reviewed >= 0),
    PRIMARY KEY (day, level_id)
);

CREATE INDEX aredl_submission_daily_level_stats_level_day_idx
    ON aredl.submission_daily_level_stats (level_id, day DESC);

CREATE TABLE arepl.submission_daily_total_stats (
    day date PRIMARY KEY,
    submitted bigint NOT NULL DEFAULT 0 CHECK (submitted >= 0),
    accepted bigint NOT NULL DEFAULT 0 CHECK (accepted >= 0),
    denied bigint NOT NULL DEFAULT 0 CHECK (denied >= 0),
    under_consideration bigint NOT NULL DEFAULT 0 CHECK (under_consideration >= 0),
    reviewed bigint NOT NULL DEFAULT 0 CHECK (reviewed >= 0)
);

CREATE TABLE arepl.submission_daily_reviewer_stats (
    day date NOT NULL,
    reviewer_id uuid NOT NULL,
    accepted bigint NOT NULL DEFAULT 0 CHECK (accepted >= 0),
    denied bigint NOT NULL DEFAULT 0 CHECK (denied >= 0),
    under_consideration bigint NOT NULL DEFAULT 0 CHECK (under_consideration >= 0),
    reviewed bigint NOT NULL DEFAULT 0 CHECK (reviewed >= 0),
    PRIMARY KEY (day, reviewer_id)
);

CREATE INDEX arepl_submission_daily_reviewer_stats_reviewer_day_idx
    ON arepl.submission_daily_reviewer_stats (reviewer_id, day DESC);

CREATE TABLE arepl.submission_daily_level_stats (
    day date NOT NULL,
    level_id uuid NOT NULL,
    submitted bigint NOT NULL DEFAULT 0 CHECK (submitted >= 0),
    accepted bigint NOT NULL DEFAULT 0 CHECK (accepted >= 0),
    denied bigint NOT NULL DEFAULT 0 CHECK (denied >= 0),
    under_consideration bigint NOT NULL DEFAULT 0 CHECK (under_consideration >= 0),
    reviewed bigint NOT NULL DEFAULT 0 CHECK (reviewed >= 0),
    PRIMARY KEY (day, level_id)
);

CREATE INDEX arepl_submission_daily_level_stats_level_day_idx
    ON arepl.submission_daily_level_stats (level_id, day DESC);

CREATE OR REPLACE FUNCTION aredl.apply_submission_daily_stats_diff(
    p_day date,
    p_reviewer_id uuid,
    p_level_id uuid,
    p_submitted bigint,
    p_accepted bigint,
    p_denied bigint,
    p_under_consideration bigint
)
RETURNS void AS
$$
BEGIN
    IF p_submitted = 0
        AND p_accepted = 0
        AND p_denied = 0
        AND p_under_consideration = 0
    THEN
        RETURN;
    END IF;

    INSERT INTO aredl.submission_daily_total_stats (
        day,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    VALUES (
        p_day,
        p_submitted,
        p_accepted,
        p_denied,
        p_under_consideration,
        p_accepted + p_denied + p_under_consideration
    )
    ON CONFLICT (day) DO UPDATE
    SET
        submitted = aredl.submission_daily_total_stats.submitted + EXCLUDED.submitted,
        accepted = aredl.submission_daily_total_stats.accepted + EXCLUDED.accepted,
        denied = aredl.submission_daily_total_stats.denied + EXCLUDED.denied,
        under_consideration = aredl.submission_daily_total_stats.under_consideration + EXCLUDED.under_consideration,
        reviewed = aredl.submission_daily_total_stats.reviewed + EXCLUDED.reviewed;

    INSERT INTO aredl.submission_daily_level_stats (
        day,
        level_id,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    VALUES (
        p_day,
        p_level_id,
        p_submitted,
        p_accepted,
        p_denied,
        p_under_consideration,
        p_accepted + p_denied + p_under_consideration
    )
    ON CONFLICT (day, level_id) DO UPDATE
    SET
        submitted = aredl.submission_daily_level_stats.submitted + EXCLUDED.submitted,
        accepted = aredl.submission_daily_level_stats.accepted + EXCLUDED.accepted,
        denied = aredl.submission_daily_level_stats.denied + EXCLUDED.denied,
        under_consideration = aredl.submission_daily_level_stats.under_consideration + EXCLUDED.under_consideration,
        reviewed = aredl.submission_daily_level_stats.reviewed + EXCLUDED.reviewed;

    IF p_reviewer_id IS NOT NULL
        AND (
            p_accepted <> 0
            OR p_denied <> 0
            OR p_under_consideration <> 0
        )
    THEN
        INSERT INTO aredl.submission_daily_reviewer_stats (
            day,
            reviewer_id,
            accepted,
            denied,
            under_consideration,
            reviewed
        )
        VALUES (
            p_day,
            p_reviewer_id,
            p_accepted,
            p_denied,
            p_under_consideration,
            p_accepted + p_denied + p_under_consideration
        )
        ON CONFLICT (day, reviewer_id) DO UPDATE
        SET
            accepted = aredl.submission_daily_reviewer_stats.accepted + EXCLUDED.accepted,
            denied = aredl.submission_daily_reviewer_stats.denied + EXCLUDED.denied,
            under_consideration = aredl.submission_daily_reviewer_stats.under_consideration + EXCLUDED.under_consideration,
            reviewed = aredl.submission_daily_reviewer_stats.reviewed + EXCLUDED.reviewed;
    END IF;

END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION arepl.apply_submission_daily_stats_diff(
    p_day date,
    p_reviewer_id uuid,
    p_level_id uuid,
    p_submitted bigint,
    p_accepted bigint,
    p_denied bigint,
    p_under_consideration bigint
)
RETURNS void AS
$$
BEGIN
    IF p_submitted = 0
        AND p_accepted = 0
        AND p_denied = 0
        AND p_under_consideration = 0
    THEN
        RETURN;
    END IF;

    INSERT INTO arepl.submission_daily_total_stats (
        day,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    VALUES (
        p_day,
        p_submitted,
        p_accepted,
        p_denied,
        p_under_consideration,
        p_accepted + p_denied + p_under_consideration
    )
    ON CONFLICT (day) DO UPDATE
    SET
        submitted = arepl.submission_daily_total_stats.submitted + EXCLUDED.submitted,
        accepted = arepl.submission_daily_total_stats.accepted + EXCLUDED.accepted,
        denied = arepl.submission_daily_total_stats.denied + EXCLUDED.denied,
        under_consideration = arepl.submission_daily_total_stats.under_consideration + EXCLUDED.under_consideration,
        reviewed = arepl.submission_daily_total_stats.reviewed + EXCLUDED.reviewed;

    INSERT INTO arepl.submission_daily_level_stats (
        day,
        level_id,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    VALUES (
        p_day,
        p_level_id,
        p_submitted,
        p_accepted,
        p_denied,
        p_under_consideration,
        p_accepted + p_denied + p_under_consideration
    )
    ON CONFLICT (day, level_id) DO UPDATE
    SET
        submitted = arepl.submission_daily_level_stats.submitted + EXCLUDED.submitted,
        accepted = arepl.submission_daily_level_stats.accepted + EXCLUDED.accepted,
        denied = arepl.submission_daily_level_stats.denied + EXCLUDED.denied,
        under_consideration = arepl.submission_daily_level_stats.under_consideration + EXCLUDED.under_consideration,
        reviewed = arepl.submission_daily_level_stats.reviewed + EXCLUDED.reviewed;

    IF p_reviewer_id IS NOT NULL
        AND (
            p_accepted <> 0
            OR p_denied <> 0
            OR p_under_consideration <> 0
        )
    THEN
        INSERT INTO arepl.submission_daily_reviewer_stats (
            day,
            reviewer_id,
            accepted,
            denied,
            under_consideration,
            reviewed
        )
        VALUES (
            p_day,
            p_reviewer_id,
            p_accepted,
            p_denied,
            p_under_consideration,
            p_accepted + p_denied + p_under_consideration
        )
        ON CONFLICT (day, reviewer_id) DO UPDATE
        SET
            accepted = arepl.submission_daily_reviewer_stats.accepted + EXCLUDED.accepted,
            denied = arepl.submission_daily_reviewer_stats.denied + EXCLUDED.denied,
            under_consideration = arepl.submission_daily_reviewer_stats.under_consideration + EXCLUDED.under_consideration,
            reviewed = arepl.submission_daily_reviewer_stats.reviewed + EXCLUDED.reviewed;
    END IF;

END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION aredl.update_submission_daily_stats_from_submission()
RETURNS TRIGGER AS
$$
DECLARE
    stats_day date;
    submitted_diff bigint;
    accepted_diff bigint;
    denied_diff bigint;
    under_consideration_diff bigint;
BEGIN
    IF TG_OP = 'UPDATE' THEN
        IF NEW IS NOT DISTINCT FROM OLD THEN
            RETURN NEW;
        END IF;

        IF aredl.submission_is_only_claim_toggle(OLD, NEW) THEN
            RETURN NEW;
        END IF;
    END IF;

    stats_day = DATE(CLOCK_TIMESTAMP());
    submitted_diff = CASE
        WHEN NEW.status = 'Pending'::submission_status
             AND (TG_OP = 'INSERT' OR OLD.status IS DISTINCT FROM 'Pending'::submission_status)
        THEN 1
        ELSE 0
    END;
    accepted_diff = (NEW.status = 'Accepted'::submission_status)::int;
    denied_diff = (NEW.status = 'Denied'::submission_status)::int;
    under_consideration_diff = (NEW.status = 'UnderConsideration'::submission_status)::int;

    PERFORM aredl.apply_submission_daily_stats_diff(
        stats_day,
        NEW.reviewer_id,
        NEW.level_id,
        submitted_diff,
        accepted_diff,
        denied_diff,
        under_consideration_diff
    );

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION arepl.update_submission_daily_stats_from_submission()
RETURNS TRIGGER AS
$$
DECLARE
    stats_day date;
    submitted_diff bigint;
    accepted_diff bigint;
    denied_diff bigint;
    under_consideration_diff bigint;
BEGIN
    IF TG_OP = 'UPDATE' THEN
        IF NEW IS NOT DISTINCT FROM OLD THEN
            RETURN NEW;
        END IF;

        IF arepl.submission_is_only_claim_toggle(OLD, NEW) THEN
            RETURN NEW;
        END IF;
    END IF;

    stats_day = DATE(CLOCK_TIMESTAMP());
    submitted_diff = CASE
        WHEN NEW.status = 'Pending'::submission_status
             AND (TG_OP = 'INSERT' OR OLD.status IS DISTINCT FROM 'Pending'::submission_status)
        THEN 1
        ELSE 0
    END;
    accepted_diff = CASE
        WHEN NEW.status = 'Accepted'::submission_status
             AND NEW.reviewer_id IS NOT NULL
        THEN 1
        ELSE 0
    END;
    denied_diff = (NEW.status = 'Denied'::submission_status)::int;
    under_consideration_diff = (NEW.status = 'UnderConsideration'::submission_status)::int;

    PERFORM arepl.apply_submission_daily_stats_diff(
        stats_day,
        NEW.reviewer_id,
        NEW.level_id,
        submitted_diff,
        accepted_diff,
        denied_diff,
        under_consideration_diff
    );

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER submission_daily_stats_ins
AFTER INSERT ON aredl.submissions
FOR EACH ROW
EXECUTE FUNCTION aredl.update_submission_daily_stats_from_submission();

CREATE TRIGGER submission_daily_stats_upd
AFTER UPDATE ON aredl.submissions
FOR EACH ROW
EXECUTE FUNCTION aredl.update_submission_daily_stats_from_submission();

CREATE TRIGGER submission_daily_stats_ins
AFTER INSERT ON arepl.submissions
FOR EACH ROW
EXECUTE FUNCTION arepl.update_submission_daily_stats_from_submission();

CREATE TRIGGER submission_daily_stats_upd
AFTER UPDATE ON arepl.submissions
FOR EACH ROW
EXECUTE FUNCTION arepl.update_submission_daily_stats_from_submission();

CREATE OR REPLACE FUNCTION aredl.rebuild_submission_daily_stats()
RETURNS void AS
$$
BEGIN
    TRUNCATE
        aredl.submission_daily_total_stats,
        aredl.submission_daily_reviewer_stats,
        aredl.submission_daily_level_stats;

    INSERT INTO aredl.submission_daily_total_stats (
        day,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    WITH hist AS (
        SELECT
            DATE(h.timestamp) AS day,
            h.submission_id,
            h.reviewer_id,
            h.status,
            h.timestamp,
            h.id,
            CASE
                WHEN h.status = 'Pending'::submission_status
                    AND LAG(h.status) OVER (
                        PARTITION BY h.submission_id
                        ORDER BY h.timestamp, h.id
                    ) = 'Pending'::submission_status
                THEN 0
                ELSE 1
            END AS pending_kept
        FROM aredl.submission_history h
        INNER JOIN aredl.submissions s ON s.id = h.submission_id
    ),
    totals AS (
        SELECT
            day,
            SUM(CASE WHEN status = 'Pending'::submission_status THEN pending_kept ELSE 0 END)::bigint AS submitted,
            SUM((status = 'Accepted'::submission_status)::int)::bigint AS accepted,
            SUM((status = 'Denied'::submission_status)::int)::bigint AS denied,
            SUM((status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
            SUM(
                (status = 'Accepted'::submission_status)::int
                + (status = 'Denied'::submission_status)::int
                + (status = 'UnderConsideration'::submission_status)::int
            )::bigint AS reviewed
        FROM hist
        GROUP BY day
    )
    SELECT day, submitted, accepted, denied, under_consideration, reviewed
    FROM totals
    WHERE submitted <> 0
       OR accepted <> 0
       OR denied <> 0
       OR under_consideration <> 0
       OR reviewed <> 0;

    INSERT INTO aredl.submission_daily_reviewer_stats (
        day,
        reviewer_id,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    SELECT
        DATE(h.timestamp) AS day,
        h.reviewer_id,
        SUM((h.status = 'Accepted'::submission_status)::int)::bigint AS accepted,
        SUM((h.status = 'Denied'::submission_status)::int)::bigint AS denied,
        SUM((h.status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
        SUM(
            (h.status = 'Accepted'::submission_status)::int
            + (h.status = 'Denied'::submission_status)::int
            + (h.status = 'UnderConsideration'::submission_status)::int
        )::bigint AS reviewed
    FROM aredl.submission_history h
    INNER JOIN aredl.submissions s ON s.id = h.submission_id
    WHERE h.reviewer_id IS NOT NULL
    GROUP BY DATE(h.timestamp), h.reviewer_id
    HAVING SUM((h.status = 'Accepted'::submission_status)::int) <> 0
        OR SUM((h.status = 'Denied'::submission_status)::int) <> 0
        OR SUM((h.status = 'UnderConsideration'::submission_status)::int) <> 0;

    INSERT INTO aredl.submission_daily_level_stats (
        day,
        level_id,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    WITH hist AS (
        SELECT
            DATE(h.timestamp) AS day,
            h.submission_id,
            s.level_id,
            h.status,
            h.timestamp,
            h.id,
            CASE
                WHEN h.status = 'Pending'::submission_status
                    AND LAG(h.status) OVER (
                        PARTITION BY h.submission_id
                        ORDER BY h.timestamp, h.id
                    ) = 'Pending'::submission_status
                THEN 0
                ELSE 1
            END AS pending_kept
        FROM aredl.submission_history h
        INNER JOIN aredl.submissions s ON s.id = h.submission_id
    ),
    totals AS (
        SELECT
            day,
            level_id,
            SUM(CASE WHEN status = 'Pending'::submission_status THEN pending_kept ELSE 0 END)::bigint AS submitted,
            SUM((status = 'Accepted'::submission_status)::int)::bigint AS accepted,
            SUM((status = 'Denied'::submission_status)::int)::bigint AS denied,
            SUM((status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
            SUM(
                (status = 'Accepted'::submission_status)::int
                + (status = 'Denied'::submission_status)::int
                + (status = 'UnderConsideration'::submission_status)::int
            )::bigint AS reviewed
        FROM hist
        GROUP BY day, level_id
    )
    SELECT day, level_id, submitted, accepted, denied, under_consideration, reviewed
    FROM totals
    WHERE submitted <> 0
       OR accepted <> 0
       OR denied <> 0
       OR under_consideration <> 0
       OR reviewed <> 0;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION arepl.rebuild_submission_daily_stats()
RETURNS void AS
$$
BEGIN
    TRUNCATE
        arepl.submission_daily_total_stats,
        arepl.submission_daily_reviewer_stats,
        arepl.submission_daily_level_stats;

    INSERT INTO arepl.submission_daily_total_stats (
        day,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    WITH hist AS (
        SELECT
            DATE(h.timestamp) AS day,
            h.submission_id,
            h.reviewer_id,
            h.status,
            h.timestamp,
            h.id,
            CASE
                WHEN h.status = 'Pending'::submission_status
                    AND LAG(h.status) OVER (
                        PARTITION BY h.submission_id
                        ORDER BY h.timestamp, h.id
                    ) = 'Pending'::submission_status
                THEN 0
                ELSE 1
            END AS pending_kept
        FROM arepl.submission_history h
        INNER JOIN arepl.submissions s ON s.id = h.submission_id
    ),
    totals AS (
        SELECT
            day,
            SUM(CASE WHEN status = 'Pending'::submission_status THEN pending_kept ELSE 0 END)::bigint AS submitted,
            SUM(
                CASE
                    WHEN status = 'Accepted'::submission_status
                         AND reviewer_id IS NOT NULL
                    THEN 1
                    ELSE 0
                END
            )::bigint AS accepted,
            SUM((status = 'Denied'::submission_status)::int)::bigint AS denied,
            SUM((status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
            SUM(
                CASE
                    WHEN status = 'Accepted'::submission_status
                         AND reviewer_id IS NOT NULL
                    THEN 1
                    ELSE 0
                END
                + (status = 'Denied'::submission_status)::int
                + (status = 'UnderConsideration'::submission_status)::int
            )::bigint AS reviewed
        FROM hist
        GROUP BY day
    )
    SELECT day, submitted, accepted, denied, under_consideration, reviewed
    FROM totals
    WHERE submitted <> 0
       OR accepted <> 0
       OR denied <> 0
       OR under_consideration <> 0
       OR reviewed <> 0;

    INSERT INTO arepl.submission_daily_reviewer_stats (
        day,
        reviewer_id,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    SELECT
        DATE(h.timestamp) AS day,
        h.reviewer_id,
        SUM((h.status = 'Accepted'::submission_status)::int)::bigint AS accepted,
        SUM((h.status = 'Denied'::submission_status)::int)::bigint AS denied,
        SUM((h.status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
        SUM(
            (h.status = 'Accepted'::submission_status)::int
            + (h.status = 'Denied'::submission_status)::int
            + (h.status = 'UnderConsideration'::submission_status)::int
        )::bigint AS reviewed
    FROM arepl.submission_history h
    INNER JOIN arepl.submissions s ON s.id = h.submission_id
    WHERE h.reviewer_id IS NOT NULL
    GROUP BY DATE(h.timestamp), h.reviewer_id
    HAVING SUM((h.status = 'Accepted'::submission_status)::int) <> 0
        OR SUM((h.status = 'Denied'::submission_status)::int) <> 0
        OR SUM((h.status = 'UnderConsideration'::submission_status)::int) <> 0;

    INSERT INTO arepl.submission_daily_level_stats (
        day,
        level_id,
        submitted,
        accepted,
        denied,
        under_consideration,
        reviewed
    )
    WITH hist AS (
        SELECT
            DATE(h.timestamp) AS day,
            h.submission_id,
            h.reviewer_id,
            s.level_id,
            h.status,
            h.timestamp,
            h.id,
            CASE
                WHEN h.status = 'Pending'::submission_status
                    AND LAG(h.status) OVER (
                        PARTITION BY h.submission_id
                        ORDER BY h.timestamp, h.id
                    ) = 'Pending'::submission_status
                THEN 0
                ELSE 1
            END AS pending_kept
        FROM arepl.submission_history h
        INNER JOIN arepl.submissions s ON s.id = h.submission_id
    ),
    totals AS (
        SELECT
            day,
            level_id,
            SUM(CASE WHEN status = 'Pending'::submission_status THEN pending_kept ELSE 0 END)::bigint AS submitted,
            SUM(
                CASE
                    WHEN status = 'Accepted'::submission_status
                         AND reviewer_id IS NOT NULL
                    THEN 1
                    ELSE 0
                END
            )::bigint AS accepted,
            SUM((status = 'Denied'::submission_status)::int)::bigint AS denied,
            SUM((status = 'UnderConsideration'::submission_status)::int)::bigint AS under_consideration,
            SUM(
                CASE
                    WHEN status = 'Accepted'::submission_status
                         AND reviewer_id IS NOT NULL
                    THEN 1
                    ELSE 0
                END
                + (status = 'Denied'::submission_status)::int
                + (status = 'UnderConsideration'::submission_status)::int
            )::bigint AS reviewed
        FROM hist
        GROUP BY day, level_id
    )
    SELECT day, level_id, submitted, accepted, denied, under_consideration, reviewed
    FROM totals
    WHERE submitted <> 0
       OR accepted <> 0
       OR denied <> 0
       OR under_consideration <> 0
       OR reviewed <> 0;
END;
$$ LANGUAGE plpgsql;

SELECT aredl.rebuild_submission_daily_stats();
SELECT arepl.rebuild_submission_daily_stats();

DROP MATERIALIZED VIEW IF EXISTS aredl.submission_stats;
DROP MATERIALIZED VIEW IF EXISTS arepl.submission_stats;
