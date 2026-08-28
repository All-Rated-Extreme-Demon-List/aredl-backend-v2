DROP MATERIALIZED VIEW IF EXISTS aredl.record_totals;

CREATE MATERIALIZED VIEW aredl.record_totals AS
SELECT NULL::uuid AS level_id,
    COUNT(*) AS records
FROM aredl.records
UNION ALL
SELECT level_id,
    COUNT(*) AS records
FROM aredl.records
GROUP BY level_id;

CREATE UNIQUE INDEX aredl_record_totals_idx
    ON aredl.record_totals (COALESCE(level_id, '00000000-0000-0000-0000-000000000000'::uuid));

DROP MATERIALIZED VIEW IF EXISTS arepl.record_totals;

CREATE MATERIALIZED VIEW arepl.record_totals AS
SELECT NULL::uuid AS level_id,
    COUNT(*) AS records
FROM arepl.records
UNION ALL
SELECT level_id,
    COUNT(*) AS records
FROM arepl.records
GROUP BY level_id;

CREATE UNIQUE INDEX arepl_record_totals_idx
    ON arepl.record_totals (COALESCE(level_id, '00000000-0000-0000-0000-000000000000'::uuid));
