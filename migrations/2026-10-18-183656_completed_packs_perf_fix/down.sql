CREATE OR REPLACE VIEW public.completed_packs AS
WITH pcl AS (
    SELECT list_id, pack_id, COUNT(*) AS lc
    FROM public.pack_levels
    GROUP BY list_id, pack_id
)
SELECT r.list_id, r.submitted_by AS user_id, pl.pack_id,
       MAX(r.achieved_at) AS completed_at
FROM public.records r
JOIN public.users u ON u.id = r.submitted_by AND u.account_status <= 2
JOIN public.pack_levels pl ON (pl.list_id, pl.level_id) = (r.list_id, r.level_id)
JOIN pcl ON (pcl.list_id, pcl.pack_id) = (pl.list_id, pl.pack_id)
GROUP BY r.list_id, r.submitted_by, pl.pack_id, pcl.lc
HAVING COUNT(*) = pcl.lc;
