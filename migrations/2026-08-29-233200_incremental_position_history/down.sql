DROP VIEW IF EXISTS aredl.badge_level_statistics;
DROP VIEW IF EXISTS arepl.badge_level_statistics;

DROP TRIGGER IF EXISTS level_place_history ON aredl.levels;
DROP TRIGGER IF EXISTS level_move ON aredl.levels;
DROP TRIGGER IF EXISTS level_place_history ON arepl.levels;
DROP TRIGGER IF EXISTS level_move ON arepl.levels;

DROP FUNCTION IF EXISTS aredl.level_place_history() CASCADE;
DROP FUNCTION IF EXISTS aredl.level_move() CASCADE;
DROP FUNCTION IF EXISTS arepl.level_place_history() CASCADE;
DROP FUNCTION IF EXISTS arepl.level_move() CASCADE;
DROP FUNCTION IF EXISTS aredl.append_position_history_full_view(INTEGER);
DROP FUNCTION IF EXISTS arepl.append_position_history_full_view(INTEGER);
DROP FUNCTION IF EXISTS aredl.rebuild_position_history_full_view();
DROP FUNCTION IF EXISTS arepl.rebuild_position_history_full_view();

DROP TABLE IF EXISTS aredl.position_history_full_view;
DROP TABLE IF EXISTS arepl.position_history_full_view;

CREATE MATERIALIZED VIEW aredl.position_history_full_view AS
WITH RECURSIVE ranked_history AS (
    SELECT ROW_NUMBER() OVER (ORDER BY i) AS i, old_position, new_position, old_status, new_status,
           COALESCE(old_status IN ('MainList', 'Legacy'), FALSE) AS old_placed,
           new_status IN ('MainList', 'Legacy') AS new_placed, created_at, affected_level
    FROM aredl.position_history
),
full_history AS (
    SELECT i, affected_level AS id, new_position AS position, CAST(NULL AS INT) AS prev_pos,
           new_status AS status, CAST(NULL AS level_status) AS prev_status,
           created_at AS action_at, affected_level AS cause, false AS moved
    FROM ranked_history
    WHERE old_status IS NULL
    UNION
    SELECT
        r.i,
        h.id,
        CASE
            WHEN r.affected_level = h.id THEN r.new_position
            WHEN h.status NOT IN ('MainList', 'Legacy') THEN h.position
            WHEN NOT r.old_placed AND r.new_placed THEN
                CASE WHEN h.position >= r.new_position THEN h.position + 1 ELSE h.position END
            WHEN r.old_placed AND NOT r.new_placed THEN
                CASE WHEN h.position > r.old_position THEN h.position - 1 ELSE h.position END
            WHEN r.old_position < r.new_position THEN
                CASE WHEN h.position BETWEEN r.old_position AND r.new_position THEN h.position - 1 ELSE h.position END
            WHEN r.old_position > r.new_position THEN
                CASE WHEN h.position BETWEEN r.new_position AND r.old_position THEN h.position + 1 ELSE h.position END
            ELSE h.position
        END AS position,
        h.position AS prev_pos,
        CASE WHEN r.affected_level = h.id THEN r.new_status ELSE h.status END AS status,
        h.status AS prev_status,
        r.created_at AS action_at,
        r.affected_level AS cause,
        (r.old_position IS NOT NULL AND r.new_position IS NOT NULL) AS moved
    FROM ranked_history r
    INNER JOIN full_history h ON r.i = h.i + 1
),
filtered AS (
    SELECT i::INTEGER AS ord, id AS affected_level, position, moved, status, action_at, cause
    FROM full_history
    WHERE prev_pos <> position OR prev_status <> status OR prev_status IS NULL
)
SELECT *, position - LAG(position, 1) OVER (PARTITION BY affected_level ORDER BY ord ASC) AS pos_diff
FROM filtered;

CREATE MATERIALIZED VIEW arepl.position_history_full_view AS
WITH RECURSIVE ranked_history AS (
    SELECT ROW_NUMBER() OVER (ORDER BY i) AS i, old_position, new_position, old_status, new_status,
           COALESCE(old_status IN ('MainList', 'Legacy'), FALSE) AS old_placed,
           new_status IN ('MainList', 'Legacy') AS new_placed, created_at, affected_level
    FROM arepl.position_history
),
full_history AS (
    SELECT i, affected_level AS id, new_position AS position, CAST(NULL AS INT) AS prev_pos,
           new_status AS status, CAST(NULL AS level_status) AS prev_status,
           created_at AS action_at, affected_level AS cause, false AS moved
    FROM ranked_history
    WHERE old_status IS NULL
    UNION
    SELECT
        r.i,
        h.id,
        CASE
            WHEN r.affected_level = h.id THEN r.new_position
            WHEN h.status NOT IN ('MainList', 'Legacy') THEN h.position
            WHEN NOT r.old_placed AND r.new_placed THEN
                CASE WHEN h.position >= r.new_position THEN h.position + 1 ELSE h.position END
            WHEN r.old_placed AND NOT r.new_placed THEN
                CASE WHEN h.position > r.old_position THEN h.position - 1 ELSE h.position END
            WHEN r.old_position < r.new_position THEN
                CASE WHEN h.position BETWEEN r.old_position AND r.new_position THEN h.position - 1 ELSE h.position END
            WHEN r.old_position > r.new_position THEN
                CASE WHEN h.position BETWEEN r.new_position AND r.old_position THEN h.position + 1 ELSE h.position END
            ELSE h.position
        END AS position,
        h.position AS prev_pos,
        CASE WHEN r.affected_level = h.id THEN r.new_status ELSE h.status END AS status,
        h.status AS prev_status,
        r.created_at AS action_at,
        r.affected_level AS cause,
        (r.old_position IS NOT NULL AND r.new_position IS NOT NULL) AS moved
    FROM ranked_history r
    INNER JOIN full_history h ON r.i = h.i + 1
),
filtered AS (
    SELECT i::INTEGER AS ord, id AS affected_level, position, moved, status, action_at, cause
    FROM full_history
    WHERE prev_pos <> position OR prev_status <> status OR prev_status IS NULL
)
SELECT *, position - LAG(position, 1) OVER (PARTITION BY affected_level ORDER BY ord ASC) AS pos_diff
FROM filtered;

CREATE INDEX aredl_position_history_full_view_affected_ord_idx
ON aredl.position_history_full_view (affected_level, ord DESC);

CREATE INDEX arepl_position_history_full_view_affected_ord_idx
ON arepl.position_history_full_view (affected_level, ord DESC);

CREATE INDEX aredl_position_history_full_view_level_action_idx
ON aredl.position_history_full_view (affected_level, action_at DESC, ord DESC)
INCLUDE (position, status);

CREATE INDEX arepl_position_history_full_view_level_action_idx
ON arepl.position_history_full_view (affected_level, action_at DESC, ord DESC)
INCLUDE (position, status);

CREATE INDEX aredl_position_history_full_view_level_first_placed_idx
ON aredl.position_history_full_view (affected_level, action_at ASC, ord ASC)
INCLUDE (position, status)
WHERE position IS NOT NULL;

CREATE INDEX arepl_position_history_full_view_level_first_placed_idx
ON arepl.position_history_full_view (affected_level, action_at ASC, ord ASC)
INCLUDE (position, status)
WHERE position IS NOT NULL;

CREATE FUNCTION aredl.level_place_history() RETURNS TRIGGER AS
$$
DECLARE
    above UUID;
    below UUID;
BEGIN
    IF NEW.status IN ('MainList', 'Legacy') THEN
        above := (
            SELECT id
            FROM aredl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position - 1
            ORDER BY id
            LIMIT 1
        );
        below := (
            SELECT id
            FROM aredl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position + 1
            ORDER BY id
            LIMIT 1
        );
    ELSE
        above := NULL;
        below := NULL;
    END IF;

    INSERT INTO aredl.position_history(new_position, old_position, old_status, new_status, affected_level, level_above, level_below)
    VALUES (NEW.position, NULL, NULL, NEW.status, NEW.id, above, below);

    REFRESH MATERIALIZED VIEW aredl.position_history_full_view;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_place_history
AFTER INSERT ON aredl.levels
FOR EACH ROW
EXECUTE PROCEDURE aredl.level_place_history();

CREATE FUNCTION aredl.level_move() RETURNS TRIGGER AS
$$
DECLARE
    old_placed BOOLEAN;
    new_placed BOOLEAN;
    above UUID;
    below UUID;
BEGIN
    IF NEW.position IS NOT DISTINCT FROM OLD.position
       AND NEW.status IS NOT DISTINCT FROM OLD.status THEN
        RETURN NULL;
    END IF;

    old_placed := OLD.status IN ('MainList', 'Legacy');
    new_placed := NEW.status IN ('MainList', 'Legacy');

    UPDATE aredl.levels
    SET position = position + CASE
        WHEN NOT old_placed AND new_placed THEN 1
        WHEN old_placed AND NOT new_placed THEN -1
        WHEN OLD.position < NEW.position THEN -1
        ELSE 1
    END
    WHERE id <> NEW.id
      AND status IN ('MainList', 'Legacy')
      AND (
          (NOT old_placed AND new_placed AND position >= NEW.position)
          OR
          (old_placed AND NOT new_placed AND position > OLD.position)
          OR
          (old_placed AND new_placed AND position BETWEEN LEAST(NEW.position, OLD.position) AND GREATEST(NEW.position, OLD.position))
      );

    IF new_placed THEN
        above := (
            SELECT id
            FROM aredl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position - 1
            ORDER BY id
            LIMIT 1
        );
        below := (
            SELECT id
            FROM aredl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position + 1
            ORDER BY id
            LIMIT 1
        );
    ELSE
        above := NULL;
        below := NULL;
    END IF;

    INSERT INTO aredl.position_history(new_position, old_position, old_status, new_status, affected_level, level_above, level_below)
    VALUES (NEW.position, OLD.position, OLD.status, NEW.status, NEW.id, above, below);

    REFRESH MATERIALIZED VIEW aredl.position_history_full_view;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_move
AFTER UPDATE OF position, status ON aredl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE aredl.level_move();

CREATE FUNCTION arepl.level_place_history() RETURNS TRIGGER AS
$$
DECLARE
    above UUID;
    below UUID;
BEGIN
    IF NEW.status IN ('MainList', 'Legacy') THEN
        above := (
            SELECT id
            FROM arepl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position - 1
            ORDER BY id
            LIMIT 1
        );
        below := (
            SELECT id
            FROM arepl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position + 1
            ORDER BY id
            LIMIT 1
        );
    ELSE
        above := NULL;
        below := NULL;
    END IF;

    INSERT INTO arepl.position_history(new_position, old_position, old_status, new_status, affected_level, level_above, level_below)
    VALUES (NEW.position, NULL, NULL, NEW.status, NEW.id, above, below);

    REFRESH MATERIALIZED VIEW arepl.position_history_full_view;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_place_history
AFTER INSERT ON arepl.levels
FOR EACH ROW
EXECUTE PROCEDURE arepl.level_place_history();

CREATE FUNCTION arepl.level_move() RETURNS TRIGGER AS
$$
DECLARE
    old_placed BOOLEAN;
    new_placed BOOLEAN;
    above UUID;
    below UUID;
BEGIN
    IF NEW.position IS NOT DISTINCT FROM OLD.position
       AND NEW.status IS NOT DISTINCT FROM OLD.status THEN
        RETURN NULL;
    END IF;

    old_placed := OLD.status IN ('MainList', 'Legacy');
    new_placed := NEW.status IN ('MainList', 'Legacy');

    UPDATE arepl.levels
    SET position = position + CASE
        WHEN NOT old_placed AND new_placed THEN 1
        WHEN old_placed AND NOT new_placed THEN -1
        WHEN OLD.position < NEW.position THEN -1
        ELSE 1
    END
    WHERE id <> NEW.id
      AND status IN ('MainList', 'Legacy')
      AND (
          (NOT old_placed AND new_placed AND position >= NEW.position)
          OR
          (old_placed AND NOT new_placed AND position > OLD.position)
          OR
          (old_placed AND new_placed AND position BETWEEN LEAST(NEW.position, OLD.position) AND GREATEST(NEW.position, OLD.position))
      );

    IF new_placed THEN
        above := (
            SELECT id
            FROM arepl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position - 1
            ORDER BY id
            LIMIT 1
        );
        below := (
            SELECT id
            FROM arepl.levels
            WHERE id <> NEW.id
              AND status IN ('MainList', 'Legacy')
              AND position = NEW.position + 1
            ORDER BY id
            LIMIT 1
        );
    ELSE
        above := NULL;
        below := NULL;
    END IF;

    INSERT INTO arepl.position_history(new_position, old_position, old_status, new_status, affected_level, level_above, level_below)
    VALUES (NEW.position, OLD.position, OLD.status, NEW.status, NEW.id, above, below);

    REFRESH MATERIALIZED VIEW arepl.position_history_full_view;

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_move
AFTER UPDATE OF position, status ON arepl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE arepl.level_move();

CREATE VIEW aredl.badge_level_statistics AS
SELECT
    r.submitted_by,
    l.id AS id,
    l.name,
    COALESCE(completed_state.position, first_placed_state.position) AS position,
    l.position AS current_position,
    l.level_id,
    l.two_player,
    l.publisher_id,
    l.edel_enjoyment,
    l.nlw_tier,
    l.tags,
    r.is_verification,
    r.achieved_at,
    COALESCE(first_record.submitted_by = r.submitted_by, false) AS is_first_victor,
    false AS is_fastest_time
FROM aredl.records r
JOIN aredl.levels l ON l.id = r.level_id
LEFT JOIN LATERAL (
    SELECT ph.position, ph.status, ph.action_at, ph.ord
    FROM aredl.position_history_full_view ph
    WHERE ph.affected_level = r.level_id
      AND ph.action_at <= r.achieved_at
    ORDER BY ph.action_at DESC, ph.ord DESC
    LIMIT 1
) completed_state ON true
LEFT JOIN LATERAL (
    SELECT ph.position, ph.status, ph.action_at, ph.ord
    FROM aredl.position_history_full_view ph
    WHERE ph.affected_level = r.level_id
      AND ph.position IS NOT NULL
      AND completed_state.position IS NULL
      AND (
          completed_state.status IS NULL
          OR (
              completed_state.status = 'Pending'
              AND ph.action_at >= r.achieved_at
          )
      )
    ORDER BY ph.action_at ASC, ph.ord ASC
    LIMIT 1
) first_placed_state ON true
LEFT JOIN LATERAL (
    SELECT fr.submitted_by
    FROM aredl.records fr
    WHERE fr.level_id = r.level_id
      AND fr.is_verification = false
    ORDER BY fr.achieved_at ASC, fr.created_at ASC, fr.id ASC
    LIMIT 1
) first_record ON true
WHERE completed_state.status IN ('MainList', 'Pending')
   OR (
       completed_state.status IS NULL
       AND first_placed_state.position IS NOT NULL
   );

CREATE VIEW arepl.badge_level_statistics AS
SELECT
    r.submitted_by,
    l.id AS id,
    l.name,
    COALESCE(completed_state.position, first_placed_state.position) AS position,
    l.position AS current_position,
    l.level_id,
    l.two_player,
    l.publisher_id,
    l.edel_enjoyment,
    l.nlw_tier,
    l.tags,
    r.is_verification,
    r.achieved_at,
    COALESCE(first_record.submitted_by = r.submitted_by, false) AS is_first_victor,
    COALESCE(fastest_record.submitted_by = r.submitted_by, false) AS is_fastest_time
FROM arepl.records r
JOIN arepl.levels l ON l.id = r.level_id
LEFT JOIN LATERAL (
    SELECT ph.position, ph.status, ph.action_at, ph.ord
    FROM arepl.position_history_full_view ph
    WHERE ph.affected_level = r.level_id
      AND ph.action_at <= r.achieved_at
    ORDER BY ph.action_at DESC, ph.ord DESC
    LIMIT 1
) completed_state ON true
LEFT JOIN LATERAL (
    SELECT ph.position, ph.status, ph.action_at, ph.ord
    FROM arepl.position_history_full_view ph
    WHERE ph.affected_level = r.level_id
      AND ph.position IS NOT NULL
      AND completed_state.position IS NULL
      AND (
          completed_state.status IS NULL
          OR (
              completed_state.status = 'Pending'
              AND ph.action_at >= r.achieved_at
          )
      )
    ORDER BY ph.action_at ASC, ph.ord ASC
    LIMIT 1
) first_placed_state ON true
LEFT JOIN LATERAL (
    SELECT fr.submitted_by
    FROM arepl.records fr
    WHERE fr.level_id = r.level_id
      AND fr.is_verification = false
    ORDER BY fr.achieved_at ASC, fr.created_at ASC, fr.id ASC
    LIMIT 1
) first_record ON true
LEFT JOIN LATERAL (
    SELECT fr.submitted_by
    FROM arepl.records fr
    WHERE fr.level_id = r.level_id
      AND fr.is_verification = false
    ORDER BY fr.completion_time ASC, fr.achieved_at ASC, fr.id ASC
    LIMIT 1
) fastest_record ON true
WHERE completed_state.status IN ('MainList', 'Pending')
   OR (
       completed_state.status IS NULL
       AND first_placed_state.position IS NOT NULL
   );
