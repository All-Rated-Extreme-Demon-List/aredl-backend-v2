DROP VIEW IF EXISTS aredl.badge_level_statistics;
DROP VIEW IF EXISTS arepl.badge_level_statistics;

DROP TRIGGER IF EXISTS level_place_history ON aredl.levels;
DROP TRIGGER IF EXISTS level_move ON aredl.levels;
DROP TRIGGER IF EXISTS level_place_history ON arepl.levels;
DROP TRIGGER IF EXISTS level_move ON arepl.levels;

DROP MATERIALIZED VIEW IF EXISTS aredl.position_history_full_view;
DROP MATERIALIZED VIEW IF EXISTS arepl.position_history_full_view;

CREATE TABLE aredl.position_history_full_view (
    ord INTEGER NOT NULL,
    affected_level UUID NOT NULL,
    position INTEGER,
    moved BOOLEAN NOT NULL,
    status level_status NOT NULL,
    action_at TIMESTAMPTZ NOT NULL,
    cause UUID NOT NULL,
    pos_diff INTEGER
);

CREATE TABLE arepl.position_history_full_view (
    ord INTEGER NOT NULL,
    affected_level UUID NOT NULL,
    position INTEGER,
    moved BOOLEAN NOT NULL,
    status level_status NOT NULL,
    action_at TIMESTAMPTZ NOT NULL,
    cause UUID NOT NULL,
    pos_diff INTEGER
);

CREATE FUNCTION aredl.rebuild_position_history_full_view() RETURNS VOID AS
$$
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('aredl'), hashtext('position_history_full_view'));

    TRUNCATE TABLE aredl.position_history_full_view;

    INSERT INTO aredl.position_history_full_view (
        ord,
        affected_level,
        position,
        moved,
        status,
        action_at,
        cause,
        pos_diff
    )
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
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION arepl.rebuild_position_history_full_view() RETURNS VOID AS
$$
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('arepl'), hashtext('position_history_full_view'));

    TRUNCATE TABLE arepl.position_history_full_view;

    INSERT INTO arepl.position_history_full_view (
        ord,
        affected_level,
        position,
        moved,
        status,
        action_at,
        cause,
        pos_diff
    )
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
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION aredl.append_position_history_full_view(history_entry_i INTEGER) RETURNS VOID AS
$$
DECLARE
    history_ord INTEGER;
    current_ord INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('aredl'), hashtext('position_history_full_view'));

    LOCK TABLE aredl.position_history_full_view IN EXCLUSIVE MODE;

    SELECT COUNT(*)::INTEGER
    INTO history_ord
    FROM aredl.position_history
    WHERE i <= history_entry_i;

    IF history_ord = 0 THEN
        RAISE EXCEPTION 'Position history entry % does not exist', history_entry_i;
    END IF;

    SELECT COALESCE(MAX(ord), 0)
    INTO current_ord
    FROM aredl.position_history_full_view;

    IF history_ord <> current_ord + 1 THEN
        RAISE EXCEPTION 'Cannot append position history ord %, current ord is %', history_ord, current_ord;
    END IF;

    INSERT INTO aredl.position_history_full_view (
        ord,
        affected_level,
        position,
        moved,
        status,
        action_at,
        cause,
        pos_diff
    )
    WITH r AS (
        SELECT
            history_ord AS ord,
            ph.old_position,
            ph.new_position,
            ph.old_status,
            ph.new_status,
            COALESCE(ph.old_status IN ('MainList', 'Legacy'), FALSE) AS old_placed,
            ph.new_status IN ('MainList', 'Legacy') AS new_placed,
            ph.created_at,
            ph.affected_level
        FROM aredl.position_history ph
        WHERE ph.i = history_entry_i
    ),
    previous_state AS (
        SELECT DISTINCT ON (phv.affected_level)
            phv.affected_level,
            phv.position,
            phv.status
        FROM aredl.position_history_full_view phv
        ORDER BY phv.affected_level, phv.ord DESC
    ),
    changed_existing AS (
        SELECT
            r.ord,
            ps.affected_level,
            CASE
                WHEN r.affected_level = ps.affected_level THEN r.new_position
                WHEN ps.status NOT IN ('MainList', 'Legacy') THEN ps.position
                WHEN NOT r.old_placed AND r.new_placed THEN
                    CASE WHEN ps.position >= r.new_position THEN ps.position + 1 ELSE ps.position END
                WHEN r.old_placed AND NOT r.new_placed THEN
                    CASE WHEN ps.position > r.old_position THEN ps.position - 1 ELSE ps.position END
                WHEN r.old_position < r.new_position THEN
                    CASE WHEN ps.position BETWEEN r.old_position AND r.new_position THEN ps.position - 1 ELSE ps.position END
                WHEN r.old_position > r.new_position THEN
                    CASE WHEN ps.position BETWEEN r.new_position AND r.old_position THEN ps.position + 1 ELSE ps.position END
                ELSE ps.position
            END AS position,
            ps.position AS prev_pos,
            CASE WHEN r.affected_level = ps.affected_level THEN r.new_status ELSE ps.status END AS status,
            ps.status AS prev_status,
            r.created_at AS action_at,
            r.affected_level AS cause,
            (r.old_position IS NOT NULL AND r.new_position IS NOT NULL) AS moved
        FROM previous_state ps
        CROSS JOIN r
    ),
    new_affected_level AS (
        SELECT
            r.ord,
            r.affected_level,
            r.new_position AS position,
            CAST(NULL AS INT) AS prev_pos,
            r.new_status AS status,
            CAST(NULL AS level_status) AS prev_status,
            r.created_at AS action_at,
            r.affected_level AS cause,
            false AS moved
        FROM r
        WHERE NOT EXISTS (
            SELECT 1
            FROM previous_state ps
            WHERE ps.affected_level = r.affected_level
        )
    ),
    filtered AS (
        SELECT *
        FROM changed_existing
        WHERE prev_pos <> position OR prev_status <> status OR prev_status IS NULL
        UNION ALL
        SELECT *
        FROM new_affected_level
    )
    SELECT
        f.ord,
        f.affected_level,
        f.position,
        f.moved,
        f.status,
        f.action_at,
        f.cause,
        f.position - prev.position AS pos_diff
    FROM filtered f
    LEFT JOIN LATERAL (
        SELECT phv.position
        FROM aredl.position_history_full_view phv
        WHERE phv.affected_level = f.affected_level
        ORDER BY phv.ord DESC
        LIMIT 1
    ) prev ON true;
END;
$$ LANGUAGE plpgsql;

CREATE FUNCTION arepl.append_position_history_full_view(history_entry_i INTEGER) RETURNS VOID AS
$$
DECLARE
    history_ord INTEGER;
    current_ord INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('arepl'), hashtext('position_history_full_view'));

    LOCK TABLE arepl.position_history_full_view IN EXCLUSIVE MODE;

    SELECT COUNT(*)::INTEGER
    INTO history_ord
    FROM arepl.position_history
    WHERE i <= history_entry_i;

    IF history_ord = 0 THEN
        RAISE EXCEPTION 'Position history entry % does not exist', history_entry_i;
    END IF;

    SELECT COALESCE(MAX(ord), 0)
    INTO current_ord
    FROM arepl.position_history_full_view;

    IF history_ord <> current_ord + 1 THEN
        RAISE EXCEPTION 'Cannot append position history ord %, current ord is %', history_ord, current_ord;
    END IF;

    INSERT INTO arepl.position_history_full_view (
        ord,
        affected_level,
        position,
        moved,
        status,
        action_at,
        cause,
        pos_diff
    )
    WITH r AS (
        SELECT
            history_ord AS ord,
            ph.old_position,
            ph.new_position,
            ph.old_status,
            ph.new_status,
            COALESCE(ph.old_status IN ('MainList', 'Legacy'), FALSE) AS old_placed,
            ph.new_status IN ('MainList', 'Legacy') AS new_placed,
            ph.created_at,
            ph.affected_level
        FROM arepl.position_history ph
        WHERE ph.i = history_entry_i
    ),
    previous_state AS (
        SELECT DISTINCT ON (phv.affected_level)
            phv.affected_level,
            phv.position,
            phv.status
        FROM arepl.position_history_full_view phv
        ORDER BY phv.affected_level, phv.ord DESC
    ),
    changed_existing AS (
        SELECT
            r.ord,
            ps.affected_level,
            CASE
                WHEN r.affected_level = ps.affected_level THEN r.new_position
                WHEN ps.status NOT IN ('MainList', 'Legacy') THEN ps.position
                WHEN NOT r.old_placed AND r.new_placed THEN
                    CASE WHEN ps.position >= r.new_position THEN ps.position + 1 ELSE ps.position END
                WHEN r.old_placed AND NOT r.new_placed THEN
                    CASE WHEN ps.position > r.old_position THEN ps.position - 1 ELSE ps.position END
                WHEN r.old_position < r.new_position THEN
                    CASE WHEN ps.position BETWEEN r.old_position AND r.new_position THEN ps.position - 1 ELSE ps.position END
                WHEN r.old_position > r.new_position THEN
                    CASE WHEN ps.position BETWEEN r.new_position AND r.old_position THEN ps.position + 1 ELSE ps.position END
                ELSE ps.position
            END AS position,
            ps.position AS prev_pos,
            CASE WHEN r.affected_level = ps.affected_level THEN r.new_status ELSE ps.status END AS status,
            ps.status AS prev_status,
            r.created_at AS action_at,
            r.affected_level AS cause,
            (r.old_position IS NOT NULL AND r.new_position IS NOT NULL) AS moved
        FROM previous_state ps
        CROSS JOIN r
    ),
    new_affected_level AS (
        SELECT
            r.ord,
            r.affected_level,
            r.new_position AS position,
            CAST(NULL AS INT) AS prev_pos,
            r.new_status AS status,
            CAST(NULL AS level_status) AS prev_status,
            r.created_at AS action_at,
            r.affected_level AS cause,
            false AS moved
        FROM r
        WHERE NOT EXISTS (
            SELECT 1
            FROM previous_state ps
            WHERE ps.affected_level = r.affected_level
        )
    ),
    filtered AS (
        SELECT *
        FROM changed_existing
        WHERE prev_pos <> position OR prev_status <> status OR prev_status IS NULL
        UNION ALL
        SELECT *
        FROM new_affected_level
    )
    SELECT
        f.ord,
        f.affected_level,
        f.position,
        f.moved,
        f.status,
        f.action_at,
        f.cause,
        f.position - prev.position AS pos_diff
    FROM filtered f
    LEFT JOIN LATERAL (
        SELECT phv.position
        FROM arepl.position_history_full_view phv
        WHERE phv.affected_level = f.affected_level
        ORDER BY phv.ord DESC
        LIMIT 1
    ) prev ON true;
END;
$$ LANGUAGE plpgsql;

SELECT aredl.rebuild_position_history_full_view();
SELECT arepl.rebuild_position_history_full_view();

CREATE UNIQUE INDEX aredl_position_history_full_view_ord_level_idx
ON aredl.position_history_full_view (ord, affected_level);

CREATE UNIQUE INDEX arepl_position_history_full_view_ord_level_idx
ON arepl.position_history_full_view (ord, affected_level);

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

CREATE OR REPLACE FUNCTION aredl.level_place_history() RETURNS TRIGGER AS
$$
DECLARE
    above UUID;
    below UUID;
    history_i INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('aredl'), hashtext('position_history_full_view'));

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
    VALUES (NEW.position, NULL, NULL, NEW.status, NEW.id, above, below)
    RETURNING i INTO history_i;

    PERFORM aredl.append_position_history_full_view(history_i);

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_place_history
AFTER INSERT ON aredl.levels
FOR EACH ROW
EXECUTE PROCEDURE aredl.level_place_history();

CREATE OR REPLACE FUNCTION aredl.level_move() RETURNS TRIGGER AS
$$
DECLARE
    old_placed BOOLEAN;
    new_placed BOOLEAN;
    above UUID;
    below UUID;
    history_i INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('aredl'), hashtext('position_history_full_view'));

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
    VALUES (NEW.position, OLD.position, OLD.status, NEW.status, NEW.id, above, below)
    RETURNING i INTO history_i;

    PERFORM aredl.append_position_history_full_view(history_i);

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_move
AFTER UPDATE OF position, status ON aredl.levels
FOR EACH ROW
WHEN (pg_trigger_depth() < 1)
EXECUTE PROCEDURE aredl.level_move();

CREATE OR REPLACE FUNCTION arepl.level_place_history() RETURNS TRIGGER AS
$$
DECLARE
    above UUID;
    below UUID;
    history_i INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('arepl'), hashtext('position_history_full_view'));

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
    VALUES (NEW.position, NULL, NULL, NEW.status, NEW.id, above, below)
    RETURNING i INTO history_i;

    PERFORM arepl.append_position_history_full_view(history_i);

    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER level_place_history
AFTER INSERT ON arepl.levels
FOR EACH ROW
EXECUTE PROCEDURE arepl.level_place_history();

CREATE OR REPLACE FUNCTION arepl.level_move() RETURNS TRIGGER AS
$$
DECLARE
    old_placed BOOLEAN;
    new_placed BOOLEAN;
    above UUID;
    below UUID;
    history_i INTEGER;
BEGIN
    PERFORM pg_advisory_xact_lock(hashtext('arepl'), hashtext('position_history_full_view'));

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
    VALUES (NEW.position, OLD.position, OLD.status, NEW.status, NEW.id, above, below)
    RETURNING i INTO history_i;

    PERFORM arepl.append_position_history_full_view(history_i);

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
