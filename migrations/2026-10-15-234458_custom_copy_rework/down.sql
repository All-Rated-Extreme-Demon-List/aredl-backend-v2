ALTER TABLE public.level_custom_copies
    DROP CONSTRAINT level_custom_copies_level_id_copy_id_key;

ALTER TYPE public.custom_id_status RENAME TO custom_id_status_new;
CREATE TYPE public.custom_id_status AS ENUM ('Published', 'Allowed', 'Banned');
ALTER TABLE public.level_custom_copies
    ALTER COLUMN status TYPE public.custom_id_status
    USING (CASE
        WHEN published AND status::text = 'Allowed' THEN 'Published'
        ELSE status::text
    END)::public.custom_id_status;
DROP TYPE public.custom_id_status_new;

ALTER TABLE public.level_custom_copies DROP COLUMN published;

ALTER TABLE public.level_custom_copies
    DROP CONSTRAINT level_custom_copies_id_types_check,
    ALTER COLUMN id_types TYPE public.custom_id_type USING id_types[1];
ALTER TABLE public.level_custom_copies RENAME COLUMN id_types TO id_type;

ALTER TYPE public.custom_id_type RENAME TO custom_id_type_new;
CREATE TYPE public.custom_id_type AS ENUM ('Bugfix', 'GlobedCopy', 'Ldm', 'Other');
ALTER TABLE public.level_custom_copies
    ALTER COLUMN id_type TYPE public.custom_id_type
    USING (CASE
        WHEN id_type::text = 'StartPos' THEN 'Other'
        WHEN id_type::text = 'BugFix' THEN 'Bugfix'
        ELSE id_type::text
    END)::public.custom_id_type;
DROP TYPE public.custom_id_type_new;
