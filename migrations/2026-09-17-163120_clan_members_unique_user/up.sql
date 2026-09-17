ALTER TABLE clan_members
    ADD CONSTRAINT clan_members_user_id_key UNIQUE (user_id),
    DROP CONSTRAINT clan_members_clan_id_user_id_key;
