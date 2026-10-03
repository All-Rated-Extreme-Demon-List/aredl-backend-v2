CREATE TEMPORARY TABLE missing_fkeys_user_recovery ON COMMIT DROP AS
WITH RECURSIVE latest_merges AS (
    SELECT DISTINCT ON (secondary_user) secondary_user, primary_user
    FROM public.merge_logs
    ORDER BY secondary_user, merged_at DESC, id DESC
),
recovery AS (
    SELECT
        m.secondary_user,
        m.primary_user,
        ARRAY[m.secondary_user, m.primary_user] AS visited
    FROM latest_merges m
    WHERE NOT EXISTS (
        SELECT 1 FROM public.users u WHERE u.id = m.secondary_user
    )
    UNION ALL
    SELECT
        r.secondary_user,
        m.primary_user,
        r.visited || m.primary_user
    FROM recovery r
    JOIN latest_merges m ON m.secondary_user = r.primary_user
    WHERE NOT EXISTS (
        SELECT 1 FROM public.users u WHERE u.id = r.primary_user
    )
      AND NOT m.primary_user = ANY(r.visited)
)
SELECT DISTINCT ON (r.secondary_user) r.secondary_user, r.primary_user
FROM recovery r
JOIN public.users u ON u.id = r.primary_user
ORDER BY r.secondary_user, cardinality(r.visited);

CREATE UNIQUE INDEX ON missing_fkeys_user_recovery (secondary_user);

DELETE FROM aredl.last_gddl_update g
WHERE NOT EXISTS (SELECT 1 FROM aredl.levels l WHERE l.id = g.id);

DELETE FROM aredl.levels_created c
WHERE NOT EXISTS (SELECT 1 FROM aredl.levels l WHERE l.id = c.level_id)
   OR NOT EXISTS (SELECT 1 FROM public.users u WHERE u.id = c.user_id);

DELETE FROM aredl.packs p
WHERE NOT EXISTS (SELECT 1 FROM aredl.pack_tiers t WHERE t.id = p.tier);

DELETE FROM aredl.pack_levels p
WHERE NOT EXISTS (SELECT 1 FROM aredl.packs pack WHERE pack.id = p.pack_id)
   OR NOT EXISTS (SELECT 1 FROM aredl.levels l WHERE l.id = p.level_id);

UPDATE aredl.levels l
SET publisher_id = m.primary_user
FROM pg_temp.missing_fkeys_user_recovery m
WHERE l.publisher_id = m.secondary_user;

ALTER TABLE aredl.submissions DISABLE TRIGGER submission_updated_at;
ALTER TABLE aredl.submissions DISABLE TRIGGER submission_log_history_upd;
ALTER TABLE aredl.submissions DISABLE TRIGGER submission_daily_stats_upd;

DELETE FROM aredl.records r
WHERE NOT EXISTS (SELECT 1 FROM aredl.levels l WHERE l.id = r.level_id)
   OR (
       NOT EXISTS (SELECT 1 FROM public.users u WHERE u.id = r.submitted_by)
       AND NOT EXISTS (
           SELECT 1 FROM pg_temp.missing_fkeys_user_recovery m
           WHERE m.secondary_user = r.submitted_by
       )
   );

DELETE FROM aredl.submissions s
WHERE NOT EXISTS (SELECT 1 FROM aredl.levels l WHERE l.id = s.level_id)
   OR (
       NOT EXISTS (SELECT 1 FROM public.users u WHERE u.id = s.submitted_by)
       AND NOT EXISTS (
           SELECT 1 FROM pg_temp.missing_fkeys_user_recovery m
           WHERE m.secondary_user = s.submitted_by
       )
   );

WITH ranked AS (
    SELECT
        r.id,
        ROW_NUMBER() OVER (
            PARTITION BY r.level_id, COALESCE(m.primary_user, r.submitted_by)
            ORDER BY (m.secondary_user IS NULL) DESC, r.created_at, r.id
        ) AS rank
    FROM aredl.records r
    LEFT JOIN pg_temp.missing_fkeys_user_recovery m
        ON m.secondary_user = r.submitted_by
)
DELETE FROM aredl.records r
USING ranked d
WHERE r.id = d.id AND d.rank > 1;

WITH ranked AS (
    SELECT
        s.id,
        ROW_NUMBER() OVER (
            PARTITION BY s.level_id, COALESCE(m.primary_user, s.submitted_by)
            ORDER BY (m.secondary_user IS NULL) DESC, s.created_at, s.id
        ) AS rank
    FROM aredl.submissions s
    LEFT JOIN pg_temp.missing_fkeys_user_recovery m
        ON m.secondary_user = s.submitted_by
)
DELETE FROM aredl.submissions s
USING ranked d
WHERE s.id = d.id AND d.rank > 1;

UPDATE aredl.records r
SET submitted_by = m.primary_user
FROM pg_temp.missing_fkeys_user_recovery m
WHERE r.submitted_by = m.secondary_user;

UPDATE aredl.submissions s
SET submitted_by = m.primary_user
FROM pg_temp.missing_fkeys_user_recovery m
WHERE s.submitted_by = m.secondary_user;

UPDATE aredl.submissions s
SET reviewer_id = (
    SELECT m.primary_user FROM pg_temp.missing_fkeys_user_recovery m
    WHERE m.secondary_user = s.reviewer_id
)
WHERE s.reviewer_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM public.users u WHERE u.id = s.reviewer_id);

UPDATE aredl.submission_history h
SET reviewer_id = (
    SELECT m.primary_user FROM pg_temp.missing_fkeys_user_recovery m
    WHERE m.secondary_user = h.reviewer_id
)
WHERE h.reviewer_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM public.users u WHERE u.id = h.reviewer_id);

ALTER TABLE aredl.submissions ENABLE TRIGGER submission_updated_at;
ALTER TABLE aredl.submissions ENABLE TRIGGER submission_log_history_upd;
ALTER TABLE aredl.submissions ENABLE TRIGGER submission_daily_stats_upd;

DELETE FROM arepl.last_gddl_update g
WHERE NOT EXISTS (SELECT 1 FROM arepl.levels l WHERE l.id = g.id);

DELETE FROM arepl.levels_created c
WHERE NOT EXISTS (SELECT 1 FROM arepl.levels l WHERE l.id = c.level_id)
   OR NOT EXISTS (SELECT 1 FROM public.users u WHERE u.id = c.user_id);

DELETE FROM arepl.packs p
WHERE NOT EXISTS (SELECT 1 FROM arepl.pack_tiers t WHERE t.id = p.tier);

DELETE FROM arepl.pack_levels p
WHERE NOT EXISTS (SELECT 1 FROM arepl.packs pack WHERE pack.id = p.pack_id)
   OR NOT EXISTS (SELECT 1 FROM arepl.levels l WHERE l.id = p.level_id);

UPDATE arepl.levels l
SET publisher_id = m.primary_user
FROM pg_temp.missing_fkeys_user_recovery m
WHERE l.publisher_id = m.secondary_user;

ALTER TABLE arepl.submissions DISABLE TRIGGER submission_updated_at;
ALTER TABLE arepl.submissions DISABLE TRIGGER submission_log_history_upd;
ALTER TABLE arepl.submissions DISABLE TRIGGER submission_daily_stats_upd;

DELETE FROM arepl.records r
WHERE NOT EXISTS (SELECT 1 FROM arepl.levels l WHERE l.id = r.level_id)
   OR (
       NOT EXISTS (SELECT 1 FROM public.users u WHERE u.id = r.submitted_by)
       AND NOT EXISTS (
           SELECT 1 FROM pg_temp.missing_fkeys_user_recovery m
           WHERE m.secondary_user = r.submitted_by
       )
   );

DELETE FROM arepl.submissions s
WHERE NOT EXISTS (SELECT 1 FROM arepl.levels l WHERE l.id = s.level_id)
   OR (
       NOT EXISTS (SELECT 1 FROM public.users u WHERE u.id = s.submitted_by)
       AND NOT EXISTS (
           SELECT 1 FROM pg_temp.missing_fkeys_user_recovery m
           WHERE m.secondary_user = s.submitted_by
       )
   );

WITH ranked AS (
    SELECT
        r.id,
        ROW_NUMBER() OVER (
            PARTITION BY r.level_id, COALESCE(m.primary_user, r.submitted_by)
            ORDER BY (m.secondary_user IS NULL) DESC, r.created_at, r.id
        ) AS rank
    FROM arepl.records r
    LEFT JOIN pg_temp.missing_fkeys_user_recovery m
        ON m.secondary_user = r.submitted_by
)
DELETE FROM arepl.records r
USING ranked d
WHERE r.id = d.id AND d.rank > 1;

WITH ranked AS (
    SELECT
        s.id,
        ROW_NUMBER() OVER (
            PARTITION BY s.level_id, COALESCE(m.primary_user, s.submitted_by)
            ORDER BY (m.secondary_user IS NULL) DESC, s.created_at, s.id
        ) AS rank
    FROM arepl.submissions s
    LEFT JOIN pg_temp.missing_fkeys_user_recovery m
        ON m.secondary_user = s.submitted_by
)
DELETE FROM arepl.submissions s
USING ranked d
WHERE s.id = d.id AND d.rank > 1;

UPDATE arepl.records r
SET submitted_by = m.primary_user
FROM pg_temp.missing_fkeys_user_recovery m
WHERE r.submitted_by = m.secondary_user;

UPDATE arepl.submissions s
SET submitted_by = m.primary_user
FROM pg_temp.missing_fkeys_user_recovery m
WHERE s.submitted_by = m.secondary_user;

UPDATE arepl.submissions s
SET reviewer_id = (
    SELECT m.primary_user FROM pg_temp.missing_fkeys_user_recovery m
    WHERE m.secondary_user = s.reviewer_id
)
WHERE s.reviewer_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM public.users u WHERE u.id = s.reviewer_id);

UPDATE arepl.submission_history h
SET reviewer_id = (
    SELECT m.primary_user FROM pg_temp.missing_fkeys_user_recovery m
    WHERE m.secondary_user = h.reviewer_id
)
WHERE h.reviewer_id IS NOT NULL
  AND NOT EXISTS (SELECT 1 FROM public.users u WHERE u.id = h.reviewer_id);

ALTER TABLE arepl.submissions ENABLE TRIGGER submission_updated_at;
ALTER TABLE arepl.submissions ENABLE TRIGGER submission_log_history_upd;
ALTER TABLE arepl.submissions ENABLE TRIGGER submission_daily_stats_upd;

UPDATE public.shifts s
SET user_id = m.primary_user
FROM pg_temp.missing_fkeys_user_recovery m
WHERE s.user_id = m.secondary_user;

DELETE FROM public.shifts s
WHERE NOT EXISTS (SELECT 1 FROM public.users u WHERE u.id = s.user_id);

UPDATE public.recurrent_shifts s
SET user_id = m.primary_user
FROM pg_temp.missing_fkeys_user_recovery m
WHERE s.user_id = m.secondary_user;

DELETE FROM public.recurrent_shifts s
WHERE NOT EXISTS (SELECT 1 FROM public.users u WHERE u.id = s.user_id);

WITH recovered AS (
    DELETE FROM aredl.submission_daily_reviewer_stats s
    USING pg_temp.missing_fkeys_user_recovery m
    WHERE s.reviewer_id = m.secondary_user
    RETURNING s.day, m.primary_user AS reviewer_id,
              s.accepted, s.denied, s.under_consideration, s.reviewed
),
combined AS (
    SELECT day, reviewer_id,
           SUM(accepted)::BIGINT AS accepted,
           SUM(denied)::BIGINT AS denied,
           SUM(under_consideration)::BIGINT AS under_consideration,
           SUM(reviewed)::BIGINT AS reviewed
    FROM recovered
    GROUP BY day, reviewer_id
)
INSERT INTO aredl.submission_daily_reviewer_stats AS s
    (day, reviewer_id, accepted, denied, under_consideration, reviewed)
SELECT day, reviewer_id, accepted, denied, under_consideration, reviewed
FROM combined
ON CONFLICT (day, reviewer_id) DO UPDATE
SET accepted = s.accepted + EXCLUDED.accepted,
    denied = s.denied + EXCLUDED.denied,
    under_consideration = s.under_consideration + EXCLUDED.under_consideration,
    reviewed = s.reviewed + EXCLUDED.reviewed;

DELETE FROM aredl.submission_daily_level_stats s
WHERE NOT EXISTS (SELECT 1 FROM aredl.levels l WHERE l.id = s.level_id);

WITH recovered AS (
    DELETE FROM arepl.submission_daily_reviewer_stats s
    USING pg_temp.missing_fkeys_user_recovery m
    WHERE s.reviewer_id = m.secondary_user
    RETURNING s.day, m.primary_user AS reviewer_id,
              s.accepted, s.denied, s.under_consideration, s.reviewed
),
combined AS (
    SELECT day, reviewer_id,
           SUM(accepted)::BIGINT AS accepted,
           SUM(denied)::BIGINT AS denied,
           SUM(under_consideration)::BIGINT AS under_consideration,
           SUM(reviewed)::BIGINT AS reviewed
    FROM recovered
    GROUP BY day, reviewer_id
)
INSERT INTO arepl.submission_daily_reviewer_stats AS s
    (day, reviewer_id, accepted, denied, under_consideration, reviewed)
SELECT day, reviewer_id, accepted, denied, under_consideration, reviewed
FROM combined
ON CONFLICT (day, reviewer_id) DO UPDATE
SET accepted = s.accepted + EXCLUDED.accepted,
    denied = s.denied + EXCLUDED.denied,
    under_consideration = s.under_consideration + EXCLUDED.under_consideration,
    reviewed = s.reviewed + EXCLUDED.reviewed;

DELETE FROM arepl.submission_daily_level_stats s
WHERE NOT EXISTS (SELECT 1 FROM arepl.levels l WHERE l.id = s.level_id);

ALTER TABLE aredl.levels
    ADD CONSTRAINT levels_publisher_id_fkey
    FOREIGN KEY (publisher_id) REFERENCES public.users(id)
    ON DELETE SET NULL ON UPDATE CASCADE;

ALTER TABLE aredl.last_gddl_update
    ADD CONSTRAINT last_gddl_update_id_fkey
    FOREIGN KEY (id) REFERENCES aredl.levels(id)
    ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE aredl.levels_created
    ADD CONSTRAINT levels_created_level_id_fkey
    FOREIGN KEY (level_id) REFERENCES aredl.levels(id)
    ON DELETE CASCADE ON UPDATE CASCADE,
    ADD CONSTRAINT levels_created_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES public.users(id)
    ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE aredl.packs
    ADD CONSTRAINT packs_tier_fkey
    FOREIGN KEY (tier) REFERENCES aredl.pack_tiers(id)
    ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE aredl.pack_levels
    ADD CONSTRAINT pack_levels_pack_id_fkey
    FOREIGN KEY (pack_id) REFERENCES aredl.packs(id)
    ON DELETE CASCADE ON UPDATE CASCADE,
    ADD CONSTRAINT pack_levels_level_id_fkey
    FOREIGN KEY (level_id) REFERENCES aredl.levels(id)
    ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE aredl.position_history
    ADD CONSTRAINT position_history_affected_level_fkey
    FOREIGN KEY (affected_level) REFERENCES aredl.levels(id)
    ON DELETE NO ACTION ON UPDATE NO ACTION,
    ADD CONSTRAINT position_history_level_above_fkey
    FOREIGN KEY (level_above) REFERENCES aredl.levels(id)
    ON DELETE SET NULL ON UPDATE CASCADE,
    ADD CONSTRAINT position_history_level_below_fkey
    FOREIGN KEY (level_below) REFERENCES aredl.levels(id)
    ON DELETE SET NULL ON UPDATE CASCADE;

ALTER TABLE aredl.records
    ADD CONSTRAINT records_level_id_fkey
    FOREIGN KEY (level_id) REFERENCES aredl.levels(id)
    ON DELETE CASCADE ON UPDATE CASCADE,
    ADD CONSTRAINT records_submitted_by_fkey
    FOREIGN KEY (submitted_by) REFERENCES public.users(id)
    ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE aredl.submissions
    ADD CONSTRAINT submissions_level_id_fkey
    FOREIGN KEY (level_id) REFERENCES aredl.levels(id)
    ON DELETE CASCADE ON UPDATE CASCADE,
    ADD CONSTRAINT submissions_submitted_by_fkey
    FOREIGN KEY (submitted_by) REFERENCES public.users(id)
    ON DELETE CASCADE ON UPDATE CASCADE,
    ADD CONSTRAINT submissions_reviewer_id_fkey
    FOREIGN KEY (reviewer_id) REFERENCES public.users(id)
    ON DELETE SET NULL ON UPDATE NO ACTION;



ALTER TABLE arepl.levels
    ADD CONSTRAINT levels_publisher_id_fkey
    FOREIGN KEY (publisher_id) REFERENCES public.users(id)
    ON DELETE SET NULL ON UPDATE CASCADE;

ALTER TABLE arepl.last_gddl_update
    ADD CONSTRAINT last_gddl_update_id_fkey
    FOREIGN KEY (id) REFERENCES arepl.levels(id)
    ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE arepl.levels_created
    ADD CONSTRAINT levels_created_level_id_fkey
    FOREIGN KEY (level_id) REFERENCES arepl.levels(id)
    ON DELETE CASCADE ON UPDATE CASCADE,
    ADD CONSTRAINT levels_created_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES public.users(id)
    ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE arepl.packs
    ADD CONSTRAINT packs_tier_fkey
    FOREIGN KEY (tier) REFERENCES arepl.pack_tiers(id)
    ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE arepl.pack_levels
    ADD CONSTRAINT pack_levels_pack_id_fkey
    FOREIGN KEY (pack_id) REFERENCES arepl.packs(id)
    ON DELETE CASCADE ON UPDATE CASCADE,
    ADD CONSTRAINT pack_levels_level_id_fkey
    FOREIGN KEY (level_id) REFERENCES arepl.levels(id)
    ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE arepl.position_history
    ADD CONSTRAINT position_history_affected_level_fkey
    FOREIGN KEY (affected_level) REFERENCES arepl.levels(id)
    ON DELETE NO ACTION ON UPDATE NO ACTION,
    ADD CONSTRAINT position_history_level_above_fkey
    FOREIGN KEY (level_above) REFERENCES arepl.levels(id)
    ON DELETE SET NULL ON UPDATE CASCADE,
    ADD CONSTRAINT position_history_level_below_fkey
    FOREIGN KEY (level_below) REFERENCES arepl.levels(id)
    ON DELETE SET NULL ON UPDATE CASCADE;

ALTER TABLE arepl.records
    ADD CONSTRAINT records_level_id_fkey
    FOREIGN KEY (level_id) REFERENCES arepl.levels(id)
    ON DELETE CASCADE ON UPDATE CASCADE,
    ADD CONSTRAINT records_submitted_by_fkey
    FOREIGN KEY (submitted_by) REFERENCES public.users(id)
    ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE arepl.submissions
    ADD CONSTRAINT submissions_level_id_fkey
    FOREIGN KEY (level_id) REFERENCES arepl.levels(id)
    ON DELETE CASCADE ON UPDATE CASCADE,
    ADD CONSTRAINT submissions_submitted_by_fkey
    FOREIGN KEY (submitted_by) REFERENCES public.users(id)
    ON DELETE CASCADE ON UPDATE CASCADE,
    ADD CONSTRAINT submissions_reviewer_id_fkey
    FOREIGN KEY (reviewer_id) REFERENCES public.users(id)
    ON DELETE SET NULL ON UPDATE NO ACTION;

ALTER TABLE arepl.submissions_enabled
    ADD CONSTRAINT submissions_enabled_moderator_fkey
    FOREIGN KEY (moderator) REFERENCES public.users(id)
    ON DELETE SET NULL ON UPDATE CASCADE;



ALTER TABLE public.shifts
    ADD CONSTRAINT shifts_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES public.users(id)
    ON DELETE NO ACTION ON UPDATE NO ACTION;

ALTER TABLE public.recurrent_shifts
    ADD CONSTRAINT recurrent_shifts_user_id_fkey
    FOREIGN KEY (user_id) REFERENCES public.users(id)
    ON DELETE NO ACTION ON UPDATE NO ACTION;

ALTER TABLE aredl.position_history_full_view
    ADD CONSTRAINT position_history_full_view_affected_level_fkey
    FOREIGN KEY (affected_level) REFERENCES aredl.levels(id)
    ON DELETE NO ACTION ON UPDATE NO ACTION,
    ADD CONSTRAINT position_history_full_view_cause_fkey
    FOREIGN KEY (cause) REFERENCES aredl.levels(id)
    ON DELETE NO ACTION ON UPDATE NO ACTION;

ALTER TABLE arepl.position_history_full_view
    ADD CONSTRAINT position_history_full_view_affected_level_fkey
    FOREIGN KEY (affected_level) REFERENCES arepl.levels(id)
    ON DELETE NO ACTION ON UPDATE NO ACTION,
    ADD CONSTRAINT position_history_full_view_cause_fkey
    FOREIGN KEY (cause) REFERENCES arepl.levels(id)
    ON DELETE NO ACTION ON UPDATE NO ACTION;

ALTER TABLE aredl.submission_history
    ADD CONSTRAINT submission_history_reviewer_id_fkey
    FOREIGN KEY (reviewer_id) REFERENCES public.users(id)
    ON DELETE NO ACTION ON UPDATE CASCADE;

ALTER TABLE aredl.submission_daily_reviewer_stats
    ADD CONSTRAINT submission_daily_reviewer_stats_reviewer_id_fkey
    FOREIGN KEY (reviewer_id) REFERENCES public.users(id)
    ON DELETE NO ACTION ON UPDATE CASCADE;

ALTER TABLE aredl.submission_daily_level_stats
    ADD CONSTRAINT submission_daily_level_stats_level_id_fkey
    FOREIGN KEY (level_id) REFERENCES aredl.levels(id)
    ON DELETE CASCADE ON UPDATE CASCADE;

ALTER TABLE arepl.submission_history
    ADD CONSTRAINT submission_history_reviewer_id_fkey
    FOREIGN KEY (reviewer_id) REFERENCES public.users(id)
    ON DELETE NO ACTION ON UPDATE CASCADE;

ALTER TABLE arepl.submission_daily_reviewer_stats
    ADD CONSTRAINT submission_daily_reviewer_stats_reviewer_id_fkey
    FOREIGN KEY (reviewer_id) REFERENCES public.users(id)
    ON DELETE NO ACTION ON UPDATE CASCADE;

ALTER TABLE arepl.submission_daily_level_stats
    ADD CONSTRAINT submission_daily_level_stats_level_id_fkey
    FOREIGN KEY (level_id) REFERENCES arepl.levels(id)
    ON DELETE CASCADE ON UPDATE CASCADE;
