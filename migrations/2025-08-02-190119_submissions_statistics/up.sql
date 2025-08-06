CREATE MATERIALIZED VIEW aredl.submission_stats AS
WITH base AS (
    SELECT DATE(created_at) AS day,
           NULL::uuid AS moderator_id,
           1 AS submitted,
           0 AS accepted,
           0 AS denied,
           0 AS under_consideration
    FROM aredl.submissions
    UNION ALL
    SELECT DATE(timestamp) AS day,
           reviewer_id,
           0 AS submitted,
           CASE WHEN status = 'Accepted'::submission_status THEN 1 ELSE 0 END AS accepted,
           CASE WHEN status = 'Denied'::submission_status THEN 1 ELSE 0 END AS denied,
           CASE WHEN status = 'UnderConsideration'::submission_status THEN 1 ELSE 0 END AS under_consideration
    FROM aredl.submission_history
)
SELECT day,
       NULL::uuid AS moderator_id,
       SUM(submitted) AS submitted,
       SUM(accepted) AS accepted,
       SUM(denied) AS denied,
       SUM(under_consideration) AS under_consideration
FROM base
GROUP BY day
UNION ALL
SELECT day,
       moderator_id,
       SUM(submitted) AS submitted,
       SUM(accepted) AS accepted,
       SUM(denied) AS denied,
       SUM(under_consideration) AS under_consideration
FROM base
WHERE moderator_id IS NOT NULL
GROUP BY day, moderator_id;

CREATE UNIQUE INDEX aredl_submission_stats_idx
    ON aredl.submission_stats (day, COALESCE(moderator_id, '00000000-0000-0000-0000-000000000000'::uuid));



CREATE MATERIALIZED VIEW arepl.submission_stats AS
WITH base AS (
    SELECT DATE(created_at) AS day,
           NULL::uuid AS moderator_id,
           1 AS submitted,
           0 AS accepted,
           0 AS denied,
           0 AS under_consideration
    FROM arepl.submissions
    UNION ALL
    SELECT DATE(timestamp) AS day,
           reviewer_id,
           0 AS submitted,
           CASE WHEN status = 'Accepted'::submission_status THEN 1 ELSE 0 END AS accepted,
           CASE WHEN status = 'Denied'::submission_status THEN 1 ELSE 0 END AS denied,
           CASE WHEN status = 'UnderConsideration'::submission_status THEN 1 ELSE 0 END AS under_consideration
    FROM arepl.submission_history
)
SELECT day,
       NULL::uuid AS moderator_id,
       SUM(submitted) AS submitted,
       SUM(accepted) AS accepted,
       SUM(denied) AS denied,
       SUM(under_consideration) AS under_consideration
FROM base
GROUP BY day
UNION ALL
SELECT day,
       moderator_id,
       SUM(submitted) AS submitted,
       SUM(accepted) AS accepted,
       SUM(denied) AS denied,
       SUM(under_consideration) AS under_consideration
FROM base
WHERE moderator_id IS NOT NULL
GROUP BY day, moderator_id;

CREATE UNIQUE INDEX arepl_submission_stats_idx
    ON arepl.submission_stats (day, COALESCE(moderator_id, '00000000-0000-0000-0000-000000000000'::uuid));
