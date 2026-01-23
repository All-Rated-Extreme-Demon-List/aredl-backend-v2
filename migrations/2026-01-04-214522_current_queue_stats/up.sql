CREATE MATERIALIZED VIEW aredl.submission_totals AS
SELECT
    NULL::uuid AS level_id,
    COUNT(*)::bigint AS submissions,
    100.00::double precision AS percent_of_queue
FROM aredl.submissions
WHERE status = 'Pending'

UNION ALL

SELECT
    level_id,
    COUNT(*)::bigint AS submissions,
    ROUND(
        (COUNT(*) * 100.0 / SUM(COUNT(*)) OVER ())::numeric,
        2
    )::double precision AS percent_of_queue
FROM aredl.submissions
WHERE status = 'Pending'
GROUP BY level_id
ORDER BY submissions DESC;

CREATE MATERIALIZED VIEW arepl.submission_totals AS
SELECT
    NULL::uuid AS level_id,
    COUNT(*)::bigint AS submissions,
    100.00::double precision AS percent_of_queue
FROM arepl.submissions
WHERE status = 'Pending'

UNION ALL

SELECT
    level_id,
    COUNT(*)::bigint AS submissions,
    ROUND(
        (COUNT(*) * 100.0 / SUM(COUNT(*)) OVER ())::numeric,
        2
    )::double precision AS percent_of_queue
FROM arepl.submissions
WHERE status = 'Pending'
GROUP BY level_id
ORDER BY submissions DESC;