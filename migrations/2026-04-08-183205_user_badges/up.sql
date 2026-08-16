CREATE TABLE user_badges (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    badge_code VARCHAR NOT NULL,
    unlocked_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    description VARCHAR NULL,
    PRIMARY KEY (user_id, badge_code)
);

CREATE INDEX idx_user_badges_user_id ON user_badges (user_id);

ALTER TABLE users
    ADD COLUMN featured_badge_code VARCHAR NULL;

CREATE OR REPLACE FUNCTION merge_users(p_primary_user uuid, p_secondary_user uuid)
RETURNS void
LANGUAGE plpgsql
AS $$
BEGIN
  IF p_primary_user = p_secondary_user THEN
    RAISE EXCEPTION 'Cannot merge a user with themselves';
  END IF;

  IF NOT EXISTS (SELECT 1 FROM users WHERE id = p_primary_user) THEN
    RAISE EXCEPTION 'Primary user % does not exist', p_primary_user;
  END IF;

  IF NOT EXISTS (SELECT 1 FROM users WHERE id = p_secondary_user) THEN
    RAISE EXCEPTION 'Secondary user % does not exist', p_secondary_user;
  END IF;

  PERFORM set_config('session_replication_role', 'replica', true);

  ----- deduplicate conflicting submissions (a submission for the same level exists for both users) -----
    WITH pairs AS (
        -- if secondary has an accepted one but not primary, keep secondary. in other cases keep primary.
        SELECT
            primary_s.level_id,
            CASE
                WHEN secondary_s.status = 'Accepted' AND primary_s.status <> 'Accepted' THEN secondary_s.id
                ELSE primary_s.id
            END AS keep_submission_id,
            CASE
                WHEN secondary_s.status = 'Accepted' AND primary_s.status <> 'Accepted' THEN primary_s.id
                ELSE secondary_s.id
            END AS discard_submission_id
        FROM aredl.submissions primary_s
        JOIN aredl.submissions secondary_s
            ON secondary_s.level_id = primary_s.level_id
        AND primary_s.submitted_by = p_primary_user
        AND secondary_s.submitted_by = p_secondary_user
        WHERE primary_s.id <> secondary_s.id
    ),
    move_history AS (
        UPDATE aredl.submission_history h
        SET submission_id = p.keep_submission_id
        FROM pairs p
        WHERE h.submission_id = p.discard_submission_id
        RETURNING 1
    ),
    delete_records AS (
        DELETE FROM aredl.records secondary_r
        USING pairs p
        WHERE secondary_r.submission_id = p.discard_submission_id
        RETURNING 1
    )
        DELETE FROM aredl.submissions s
        USING pairs p
        WHERE s.id = p.discard_submission_id;

  ----- same for arepl
    WITH pairs AS (
        SELECT
            primary_s.level_id,
            CASE
                WHEN secondary_s.status = 'Accepted' AND primary_s.status <> 'Accepted' THEN secondary_s.id
                ELSE primary_s.id
            END AS keep_submission_id,
            CASE
                WHEN secondary_s.status = 'Accepted' AND primary_s.status <> 'Accepted' THEN primary_s.id
                ELSE secondary_s.id
            END AS discard_submission_id
        FROM arepl.submissions primary_s
        JOIN arepl.submissions secondary_s
            ON secondary_s.level_id = primary_s.level_id
        AND primary_s.submitted_by = p_primary_user
        AND secondary_s.submitted_by = p_secondary_user
        WHERE primary_s.id <> secondary_s.id
    ),
    move_history AS (
        UPDATE arepl.submission_history h
        SET submission_id = p.keep_submission_id
        FROM pairs p
        WHERE h.submission_id = p.discard_submission_id
        RETURNING 1
    ),
    delete_records AS (
        DELETE FROM arepl.records secondary_r
        USING pairs p
        WHERE secondary_r.submission_id = p.discard_submission_id
        RETURNING 1
    )
        DELETE FROM arepl.submissions s
        USING pairs p
        WHERE s.id = p.discard_submission_id;

  ----- other deduplication

    DELETE FROM aredl.levels_created ac1
    USING aredl.levels_created ac2
    WHERE ac1.user_id = p_secondary_user
        AND ac1.level_id = ac2.level_id
        AND ac2.user_id = p_primary_user;

    DELETE FROM arepl.levels_created ac1
    USING arepl.levels_created ac2
    WHERE ac1.user_id = p_secondary_user
        AND ac1.level_id = ac2.level_id
        AND ac2.user_id = p_primary_user;

    DELETE FROM clan_members cm1
    USING clan_members cm2
    WHERE cm1.user_id = p_secondary_user
        AND cm2.user_id = p_primary_user;

    DELETE FROM user_roles ur1
    USING user_roles ur2
    WHERE ur1.user_id = p_secondary_user
        AND ur1.role_id = ur2.role_id
        AND ur2.user_id = p_primary_user;

    DELETE FROM user_badges ub1
    USING user_badges ub2
    WHERE ub1.user_id = p_secondary_user
        AND ub1.badge_code = ub2.badge_code
        AND ub2.user_id = p_primary_user;

  ----- change ownership

    UPDATE aredl.submissions SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
    UPDATE aredl.records SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
    UPDATE aredl.levels_created SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE aredl.levels SET publisher_id = p_primary_user WHERE publisher_id = p_secondary_user;

    UPDATE arepl.submissions SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
    UPDATE arepl.records SET submitted_by = p_primary_user WHERE submitted_by = p_secondary_user;
    UPDATE arepl.levels_created SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE arepl.levels SET publisher_id = p_primary_user WHERE publisher_id = p_secondary_user;

    UPDATE clan_members SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE user_roles SET user_id = p_primary_user WHERE user_id = p_secondary_user;
    UPDATE user_badges SET user_id = p_primary_user WHERE user_id = p_secondary_user;

    UPDATE users
    SET featured_badge_code = secondary_user.featured_badge_code
    FROM users AS secondary_user
    WHERE users.id = p_primary_user
      AND secondary_user.id = p_secondary_user
      AND users.featured_badge_code IS NULL
      AND secondary_user.featured_badge_code IS NOT NULL;

    PERFORM set_config('session_replication_role', 'origin', true);

  ----- log and delete

    INSERT INTO merge_logs (primary_user, secondary_user, secondary_username, secondary_discord_id, secondary_global_name)
    SELECT p_primary_user, p_secondary_user, username, discord_id, global_name
    FROM users WHERE id = p_secondary_user;

    UPDATE merge_logs SET primary_user = p_primary_user WHERE primary_user = p_secondary_user;

    DELETE FROM users WHERE id = p_secondary_user;
END;
$$;
