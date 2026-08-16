DROP MATERIALIZED VIEW IF EXISTS aredl.submission_stats;
CREATE MATERIALIZED VIEW aredl.submission_stats AS
WITH hist AS (
  SELECT
    DATE(h.timestamp) AS day,
    h.submission_id,
    h.reviewer_id,
    h.status,
    h.timestamp,
    h.id,
    CASE
      WHEN h.status = 'Pending'::submission_status
       AND LAG(h.status) OVER (
             PARTITION BY h.submission_id
             ORDER BY h.timestamp, h.id
           ) = 'Pending'::submission_status
      THEN 0
      ELSE 1
    END AS pending_kept
  FROM aredl.submission_history h
),
days AS (
  SELECT DISTINCT day
  FROM hist
),
reviewers AS (
  SELECT DISTINCT reviewer_id
  FROM hist
  WHERE reviewer_id IS NOT NULL
),
by_reviewer AS (
  SELECT
    day,
    reviewer_id,
    SUM((status = 'Accepted'::submission_status)::int)            AS accepted,
    SUM((status = 'Denied'::submission_status)::int)              AS denied,
    SUM((status = 'UnderConsideration'::submission_status)::int)  AS under_consideration
  FROM hist
  WHERE reviewer_id IS NOT NULL
  GROUP BY day, reviewer_id
),
totals AS (
  SELECT
    day,
    NULL::uuid AS reviewer_id,
    SUM(CASE WHEN status = 'Pending'::submission_status THEN pending_kept ELSE 0 END) AS submitted,
    SUM((status = 'Accepted'::submission_status)::int)            AS accepted,
    SUM((status = 'Denied'::submission_status)::int)              AS denied,
    SUM((status = 'UnderConsideration'::submission_status)::int)  AS under_consideration
  FROM hist
  GROUP BY day
)
SELECT day, reviewer_id, submitted, accepted, denied, under_consideration
FROM totals
UNION ALL
SELECT
  days.day,
  reviewers.reviewer_id,
  0 AS submitted,
  COALESCE(by_reviewer.accepted, 0) AS accepted,
  COALESCE(by_reviewer.denied, 0) AS denied,
  COALESCE(by_reviewer.under_consideration, 0) AS under_consideration
FROM days
CROSS JOIN reviewers
LEFT JOIN by_reviewer
  ON by_reviewer.day = days.day
 AND by_reviewer.reviewer_id = reviewers.reviewer_id;

CREATE UNIQUE INDEX aredl_submission_stats_idx
  ON aredl.submission_stats (day, COALESCE(reviewer_id, '00000000-0000-0000-0000-000000000000'::uuid));

CREATE INDEX aredl_submission_stats_reviewer_day_idx
  ON aredl.submission_stats (reviewer_id, day DESC);

DROP MATERIALIZED VIEW IF EXISTS arepl.submission_stats;
CREATE MATERIALIZED VIEW arepl.submission_stats AS
WITH hist AS (
  SELECT
    DATE(h.timestamp) AS day,
    h.submission_id,
    h.reviewer_id,
    h.status,
    h.timestamp,
    h.id,
    CASE
      WHEN h.status = 'Pending'::submission_status
       AND LAG(h.status) OVER (
             PARTITION BY h.submission_id
             ORDER BY h.timestamp, h.id
           ) = 'Pending'::submission_status
      THEN 0
      ELSE 1
    END AS pending_kept
  FROM arepl.submission_history h
),
days AS (
  SELECT DISTINCT day
  FROM hist
),
reviewers AS (
  SELECT DISTINCT reviewer_id
  FROM hist
  WHERE reviewer_id IS NOT NULL
),
by_reviewer AS (
  SELECT
    day,
    reviewer_id,
    SUM((status = 'Accepted'::submission_status)::int)            AS accepted,
    SUM((status = 'Denied'::submission_status)::int)              AS denied,
    SUM((status = 'UnderConsideration'::submission_status)::int)  AS under_consideration
  FROM hist
  WHERE reviewer_id IS NOT NULL
  GROUP BY day, reviewer_id
),
totals AS (
  SELECT
    day,
    NULL::uuid AS reviewer_id,
    SUM(CASE WHEN status = 'Pending'::submission_status THEN pending_kept ELSE 0 END) AS submitted,
    SUM(CASE WHEN status = 'Accepted'::submission_status AND reviewer_id IS NOT NULL THEN 1 ELSE 0 END) AS accepted,
    SUM((status = 'Denied'::submission_status)::int)              AS denied,
    SUM((status = 'UnderConsideration'::submission_status)::int)  AS under_consideration
  FROM hist
  GROUP BY day
)
SELECT day, reviewer_id, submitted, accepted, denied, under_consideration
FROM totals
UNION ALL
SELECT
  days.day,
  reviewers.reviewer_id,
  0 AS submitted,
  COALESCE(by_reviewer.accepted, 0) AS accepted,
  COALESCE(by_reviewer.denied, 0) AS denied,
  COALESCE(by_reviewer.under_consideration, 0) AS under_consideration
FROM days
CROSS JOIN reviewers
LEFT JOIN by_reviewer
  ON by_reviewer.day = days.day
 AND by_reviewer.reviewer_id = reviewers.reviewer_id;

CREATE UNIQUE INDEX arepl_submission_stats_idx
  ON arepl.submission_stats (day, COALESCE(reviewer_id, '00000000-0000-0000-0000-000000000000'::uuid));

CREATE INDEX arepl_submission_stats_reviewer_day_idx
  ON arepl.submission_stats (reviewer_id, day DESC);