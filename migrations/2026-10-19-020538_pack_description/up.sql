ALTER TABLE public.packs ADD COLUMN description TEXT;

CREATE OR REPLACE VIEW public.packs_points AS
SELECT p.list_id, p.id, p.name, p.tier, ROUND(SUM(l.points)::numeric * 0.5)::integer AS points,
       p.description
FROM public.packs p
JOIN public.pack_levels pl ON (pl.list_id, pl.pack_id) = (p.list_id, p.id)
JOIN public.levels l ON (l.list_id, l.id) = (pl.list_id, pl.level_id)
GROUP BY p.list_id, p.id;
