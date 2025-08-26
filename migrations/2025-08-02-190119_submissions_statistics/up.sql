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
SELECT r.day, r.reviewer_id, 0 AS submitted, r.accepted, r.denied, r.under_consideration
FROM by_reviewer r;

CREATE UNIQUE INDEX aredl_submission_stats_idx
  ON aredl.submission_stats (day, COALESCE(reviewer_id, '00000000-0000-0000-0000-000000000000'::uuid));

CREATE INDEX IF NOT EXISTS aredl_hist_sub_ts_id_idx ON aredl.submission_history (submission_id, timestamp, id);
CREATE INDEX IF NOT EXISTS aredl_hist_ts_idx        ON aredl.submission_history (timestamp);
CREATE INDEX IF NOT EXISTS aredl_hist_rev_ts_idx    ON aredl.submission_history (reviewer_id, timestamp);

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
SELECT r.day, r.reviewer_id, 0 AS submitted, r.accepted, r.denied, r.under_consideration
FROM by_reviewer r;

CREATE UNIQUE INDEX arepl_submission_stats_idx
  ON arepl.submission_stats (day, COALESCE(reviewer_id, '00000000-0000-0000-0000-000000000000'::uuid));

CREATE INDEX IF NOT EXISTS arepl_hist_sub_ts_id_idx ON arepl.submission_history (submission_id, timestamp, id);
CREATE INDEX IF NOT EXISTS arepl_hist_ts_idx        ON arepl.submission_history (timestamp);
CREATE INDEX IF NOT EXISTS arepl_hist_rev_ts_idx    ON arepl.submission_history (reviewer_id, timestamp);