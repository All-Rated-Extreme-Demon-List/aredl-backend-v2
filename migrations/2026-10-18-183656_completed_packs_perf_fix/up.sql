CREATE OR REPLACE VIEW public.completed_packs AS
WITH completed_levels AS (
    SELECT r.list_id, r.submitted_by AS user_id, pl.pack_id,
           COUNT(*) AS completed_count, MAX(r.achieved_at) AS completed_at
    FROM public.records r
    JOIN public.users u ON u.id = r.submitted_by AND u.account_status <= 2
    JOIN public.pack_levels pl ON (pl.list_id, pl.level_id) = (r.list_id, r.level_id)
    GROUP BY r.list_id, r.submitted_by, pl.pack_id
), pack_sizes AS (
    SELECT list_id, pack_id, COUNT(*) AS level_count
    FROM public.pack_levels
    GROUP BY list_id, pack_id
)
SELECT c.list_id, c.user_id, c.pack_id, c.completed_at
FROM completed_levels c
JOIN pack_sizes p ON (p.list_id, p.pack_id) = (c.list_id, c.pack_id)
WHERE c.completed_count = p.level_count;
