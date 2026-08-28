DROP MATERIALIZED VIEW IF EXISTS aredl.record_totals;

CREATE MATERIALIZED VIEW aredl.record_totals AS
SELECT NULL::uuid AS level_id,
    COUNT(*) FILTER (WHERE r.is_verification = false) AS records,
    COUNT(*) FILTER (WHERE r.is_verification = true) AS verifications
FROM aredl.records r
JOIN users u ON u.id = r.submitted_by AND u.ban_level <= 2
UNION ALL
SELECT r.level_id,
    COUNT(*) FILTER (WHERE r.is_verification = false) AS records,
    COUNT(*) FILTER (WHERE r.is_verification = true) AS verifications
FROM aredl.records r
JOIN users u ON u.id = r.submitted_by AND u.ban_level <= 2
GROUP BY r.level_id;

CREATE UNIQUE INDEX aredl_record_totals_idx
    ON aredl.record_totals (COALESCE(level_id, '00000000-0000-0000-0000-000000000000'::uuid));

DROP MATERIALIZED VIEW IF EXISTS arepl.record_totals;

CREATE MATERIALIZED VIEW arepl.record_totals AS
SELECT NULL::uuid AS level_id,
    COUNT(*) FILTER (WHERE r.is_verification = false) AS records,
    COUNT(*) FILTER (WHERE r.is_verification = true) AS verifications
FROM arepl.records r
JOIN users u ON u.id = r.submitted_by AND u.ban_level <= 2
UNION ALL
SELECT r.level_id,
    COUNT(*) FILTER (WHERE r.is_verification = false) AS records,
    COUNT(*) FILTER (WHERE r.is_verification = true) AS verifications
FROM arepl.records r
JOIN users u ON u.id = r.submitted_by AND u.ban_level <= 2
GROUP BY r.level_id;

CREATE UNIQUE INDEX arepl_record_totals_idx
    ON arepl.record_totals (COALESCE(level_id, '00000000-0000-0000-0000-000000000000'::uuid));
