CREATE OR REPLACE VIEW aredl.completed_packs AS
    WITH pcl AS (
        SELECT pl.pack_id, COUNT(*) AS lc
        FROM aredl.pack_levels pl
        GROUP BY pl.pack_id
    )
    SELECT
        r.submitted_by AS user_id,
        pl.pack_id,
        MAX(r.achieved_at) AS completed_at
    FROM aredl.records r
    JOIN aredl.pack_levels pl ON pl.level_id = r.level_id
    JOIN pcl ON pcl.pack_id = pl.pack_id
    GROUP BY r.submitted_by, pl.pack_id, pcl.lc
    HAVING COUNT(*) = pcl.lc;

CREATE OR REPLACE VIEW arepl.completed_packs AS
    WITH pcl AS (
        SELECT pl.pack_id, COUNT(*) AS lc
        FROM arepl.pack_levels pl
        GROUP BY pl.pack_id
    )
    SELECT
        r.submitted_by AS user_id,
        pl.pack_id,
        MAX(r.achieved_at) AS completed_at
    FROM arepl.records r
    JOIN arepl.pack_levels pl ON pl.level_id = r.level_id
    JOIN pcl ON pcl.pack_id = pl.pack_id
    GROUP BY r.submitted_by, pl.pack_id, pcl.lc
    HAVING COUNT(*) = pcl.lc;