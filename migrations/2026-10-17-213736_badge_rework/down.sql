DROP INDEX IF EXISTS public.idx_user_badges_record_id;

ALTER TABLE public.user_badges
    DROP CONSTRAINT user_badges_level_ids_no_nulls,
    DROP COLUMN level_ids,
    DROP COLUMN record_id,
    ADD COLUMN description VARCHAR NULL;

DROP VIEW public.badge_level_statistics;

CREATE VIEW public.badge_level_statistics AS
SELECT r.list_id, r.submitted_by, l.id, l.name,
       COALESCE(completed_state.position, first_placed_state.position) AS position,
       l.position AS current_position, l.level_id, l.two_player, l.publisher_id,
       l.edel_enjoyment, l.nlw_tier, l.tags, r.is_verification, r.achieved_at,
       COALESCE(first_record.submitted_by = r.submitted_by, false) AS is_first_victor,
       COALESCE(fastest_record.submitted_by = r.submitted_by, false) AS is_fastest_time
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
