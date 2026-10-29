CREATE OR REPLACE VIEW public.badge_level_statistics AS
SELECT r.list_id, r.submitted_by, l.id, l.name,
       COALESCE(completed_state.position, first_placed_state.position) AS position,
       l.position AS current_position, l.level_id, l.two_player, l.publisher_id,
       l.edel_enjoyment, l.nlw_tier, ARRAY(
           SELECT t.name FROM level_tags a
           JOIN tags t ON (t.list_id, t.id) = (a.list_id, a.tag_id)
           WHERE (a.list_id, a.level_id) = (l.list_id, l.id)
           ORDER BY t.name, t.id
       ) AS tags, r.is_verification, r.achieved_at,
       COALESCE(first_record.submitted_by = r.submitted_by, false) AS is_first_victor,
       COALESCE(fastest_record.submitted_by = r.submitted_by, false) AS is_fastest_time,
       r.id AS record_id
FROM records r
JOIN levels l ON (l.list_id, l.id) = (r.list_id, r.level_id)
LEFT JOIN LATERAL (
    SELECT ph.position, ph.status, ph.action_at, ph.ord
    FROM position_history_full_view ph
    WHERE ph.list_id = r.list_id AND ph.affected_level = r.level_id AND ph.action_at <= r.achieved_at
    ORDER BY ph.action_at DESC, ph.ord DESC LIMIT 1
) completed_state ON true
LEFT JOIN LATERAL (
    SELECT ph.position, ph.status, ph.action_at, ph.ord
    FROM position_history_full_view ph
    WHERE ph.list_id = r.list_id AND ph.affected_level = r.level_id AND ph.position IS NOT NULL
      AND completed_state.position IS NULL
      AND (completed_state.status IS NULL OR
           (completed_state.status = 'Pending' AND ph.action_at >= r.achieved_at))
    ORDER BY ph.action_at, ph.ord LIMIT 1
) first_placed_state ON true
LEFT JOIN LATERAL (
    SELECT fr.submitted_by FROM records fr
    WHERE fr.list_id = r.list_id AND fr.level_id = r.level_id AND fr.is_verification = false
    ORDER BY fr.achieved_at, fr.created_at, fr.id LIMIT 1
) first_record ON true
LEFT JOIN LATERAL (
    SELECT fr.submitted_by FROM records fr
    WHERE r.list_id = 2 AND fr.list_id = r.list_id
      AND fr.level_id = r.level_id AND fr.is_verification = false
    ORDER BY fr.completion_time, fr.achieved_at, fr.id LIMIT 1
) fastest_record ON true
WHERE completed_state.status IN ('MainList', 'Pending')
   OR (completed_state.status IS NULL AND first_placed_state.position IS NOT NULL);

DROP INDEX public.records_fastest_time_idx;
CREATE INDEX records_fastest_time_idx ON public.records
    (list_id, level_id, completion_time, achieved_at, id) INCLUDE (submitted_by)
    WHERE is_verification = false;
