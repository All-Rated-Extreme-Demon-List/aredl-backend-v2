DROP VIEW public.min_placement_country_records;
DROP VIEW public.min_placement_clans_records;

CREATE VIEW public.min_placement_country_records AS
WITH ranked AS (
    SELECT
        r.list_id,
        r.id,
        u.country,
        ROW_NUMBER() OVER record_window AS order_pos,
        COUNT(*) OVER record_window AS completion_count
    FROM public.records r
    JOIN public.users u ON u.id = r.submitted_by AND u.account_status <= 1
    JOIN public.levels l ON (l.list_id, l.id) = (r.list_id, r.level_id)
    WHERE l.status <> 'Removed' AND u.country IS NOT NULL
    WINDOW record_window AS (
        PARTITION BY r.list_id, r.level_id, u.country
        ORDER BY r.achieved_at
        ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING
    )
)
SELECT list_id, id, country, completion_count
FROM ranked
WHERE order_pos = 1;

CREATE VIEW public.min_placement_clans_records AS
WITH ranked AS (
    SELECT
        r.list_id,
        r.id,
        cm.clan_id,
        ROW_NUMBER() OVER record_window AS order_pos,
        COUNT(*) OVER record_window AS completion_count
    FROM public.records r
    JOIN public.users u ON u.id = r.submitted_by AND u.account_status <= 1
    JOIN public.clan_members cm ON cm.user_id = r.submitted_by
    JOIN public.levels l ON (l.list_id, l.id) = (r.list_id, r.level_id)
    WHERE l.status <> 'Removed'
    WINDOW record_window AS (
        PARTITION BY r.list_id, r.level_id, cm.clan_id
        ORDER BY r.achieved_at
        ROWS BETWEEN UNBOUNDED PRECEDING AND UNBOUNDED FOLLOWING
    )
)
SELECT list_id, id, clan_id, completion_count
FROM ranked
WHERE order_pos = 1;
