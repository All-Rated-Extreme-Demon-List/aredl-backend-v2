ALTER TABLE public.user_badges
    DROP COLUMN description,
    ADD COLUMN level_ids UUID[] NOT NULL DEFAULT '{}'::UUID[],
    ADD COLUMN record_id UUID REFERENCES public.records(id) ON DELETE SET NULL,
    ADD CONSTRAINT user_badges_level_ids_no_nulls CHECK (array_position(level_ids, NULL) IS NULL);

CREATE INDEX idx_user_badges_record_id ON public.user_badges(record_id) WHERE record_id IS NOT NULL;


CREATE OR REPLACE VIEW public.badge_level_statistics AS
SELECT r.list_id, r.submitted_by, l.id, l.name,
       COALESCE(completed_state.position, first_placed_state.position) AS position,
       l.position AS current_position, l.level_id, l.two_player, l.publisher_id,
       l.edel_enjoyment, l.nlw_tier, l.tags, r.is_verification, r.achieved_at,
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


UPDATE public.user_badges b
SET level_ids = ARRAY(
    SELECT l.id
    FROM public.levels l
    WHERE l.status <> 'Removed'
      AND (l.publisher_id = b.user_id OR EXISTS (
          SELECT 1 FROM public.levels_created c
          WHERE c.level_id = l.id AND c.user_id = b.user_id
      ))
    ORDER BY l.position NULLS LAST, l.name, l.id
)
WHERE b.badge_code = 'global.creator';

UPDATE public.user_badges b
SET level_ids = ARRAY(
    SELECT s.id
    FROM public.badge_level_statistics s
    WHERE s.submitted_by = b.user_id
      AND CASE b.badge_code
          WHEN 'global.verifier' THEN s.is_verification
          WHEN 'global.first_victor' THEN s.is_first_victor
          WHEN 'platformer.fastest_time' THEN s.list_id = 2 AND s.is_fastest_time
          ELSE false
      END
    ORDER BY s.position NULLS LAST, s.name, s.id
)
WHERE b.badge_code IN ('global.verifier', 'global.first_victor', 'platformer.fastest_time');

WITH publisher_groups AS (
    SELECT submitted_by, publisher_id, COUNT(*) AS completions, MIN(position) AS best_position
    FROM public.badge_level_statistics
    GROUP BY submitted_by, publisher_id
), best_publishers AS (
    SELECT DISTINCT ON (submitted_by) submitted_by, publisher_id
    FROM publisher_groups
    ORDER BY submitted_by, completions DESC, best_position NULLS FIRST, publisher_id
)
UPDATE public.user_badges b
SET level_ids = ARRAY(
    SELECT s.id
    FROM public.badge_level_statistics s
    JOIN best_publishers p ON p.submitted_by = s.submitted_by AND p.publisher_id = s.publisher_id
    WHERE s.submitted_by = b.user_id
    ORDER BY s.position NULLS LAST, s.name, s.id
)
WHERE b.badge_code IN ('global.publisher_levels.4', 'global.publisher_levels.8');