-- Replace the view without dropping its dependent leaderboard views.
ALTER VIEW public.packs_points RENAME TO packs_points_with_description;

CREATE VIEW public.packs_points AS
SELECT p.list_id, p.id, p.name, p.tier, ROUND(SUM(l.points)::numeric * 0.5)::integer AS points
FROM public.packs p
JOIN public.pack_levels pl ON (pl.list_id, pl.pack_id) = (p.list_id, p.id)
JOIN public.levels l ON (l.list_id, l.id) = (pl.list_id, pl.level_id)
GROUP BY p.list_id, p.id;

CREATE OR REPLACE VIEW public.user_pack_points AS
SELECT cp.list_id, cp.user_id, SUM(p.points)::integer AS points
FROM public.completed_packs cp
JOIN public.packs_points p ON (p.list_id, p.id) = (cp.list_id, cp.pack_id)
GROUP BY cp.list_id, cp.user_id;

DROP VIEW public.packs_points_with_description;

ALTER TABLE public.packs DROP COLUMN description;
