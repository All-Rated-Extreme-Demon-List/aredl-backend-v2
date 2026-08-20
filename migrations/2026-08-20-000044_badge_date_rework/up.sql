CREATE OR REPLACE VIEW aredl.badge_level_statistics AS
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

CREATE OR REPLACE VIEW arepl.badge_level_statistics AS
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

CREATE INDEX IF NOT EXISTS aredl_position_history_full_view_level_action_idx
ON aredl.position_history_full_view (affected_level, action_at DESC, ord DESC)
INCLUDE (position, status);

CREATE INDEX IF NOT EXISTS arepl_position_history_full_view_level_action_idx
ON arepl.position_history_full_view (affected_level, action_at DESC, ord DESC)
INCLUDE (position, status);

CREATE INDEX IF NOT EXISTS aredl_position_history_full_view_level_first_placed_idx
ON aredl.position_history_full_view (affected_level, action_at ASC, ord ASC)
INCLUDE (position, status)
WHERE position IS NOT NULL;

CREATE INDEX IF NOT EXISTS arepl_position_history_full_view_level_first_placed_idx
ON arepl.position_history_full_view (affected_level, action_at ASC, ord ASC)
INCLUDE (position, status)
WHERE position IS NOT NULL;

CREATE INDEX IF NOT EXISTS aredl_records_first_victor_idx
ON aredl.records (level_id, achieved_at, created_at, id)
INCLUDE (submitted_by)
WHERE is_verification = false;

CREATE INDEX IF NOT EXISTS arepl_records_first_victor_idx
ON arepl.records (level_id, achieved_at, created_at, id)
INCLUDE (submitted_by)
WHERE is_verification = false;

CREATE INDEX IF NOT EXISTS arepl_records_fastest_time_idx
ON arepl.records (level_id, completion_time, achieved_at, id)
INCLUDE (submitted_by)
WHERE is_verification = false;
