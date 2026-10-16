ALTER TABLE public.level_custom_copies
    ADD COLUMN published boolean NOT NULL DEFAULT false;

UPDATE public.level_custom_copies
SET published = true, status = 'Allowed'
WHERE status = 'Published';

ALTER TYPE public.custom_id_status RENAME TO custom_id_status_old;
CREATE TYPE public.custom_id_status AS ENUM ('Allowed', 'Banned');
ALTER TABLE public.level_custom_copies
    ALTER COLUMN status TYPE public.custom_id_status
    USING status::text::public.custom_id_status;
DROP TYPE public.custom_id_status_old;

ALTER TABLE public.level_custom_copies
    ADD CONSTRAINT level_custom_copies_level_id_copy_id_key UNIQUE (level_id, copy_id);

ALTER TYPE public.custom_id_type RENAME TO custom_id_type_old;
CREATE TYPE public.custom_id_type AS ENUM ('BugFix', 'GlobedCopy', 'Ldm', 'Other', 'StartPos');
ALTER TABLE public.level_custom_copies
    ALTER COLUMN id_type TYPE public.custom_id_type
    USING (CASE
        WHEN id_type::text = 'Bugfix' THEN 'BugFix'
        ELSE id_type::text
    END)::public.custom_id_type;
DROP TYPE public.custom_id_type_old;

ALTER TABLE public.level_custom_copies RENAME COLUMN id_type TO id_types;
ALTER TABLE public.level_custom_copies
    ALTER COLUMN id_types TYPE public.custom_id_type[] USING ARRAY[id_types],
    ADD CONSTRAINT level_custom_copies_id_types_check CHECK (
        cardinality(id_types) > 0
        AND array_ndims(id_types) = 1
        AND array_position(id_types, NULL) IS NULL
    );

-- backfill some old startpos copies to the new type
UPDATE public.level_custom_copies
SET id_types = ARRAY['StartPos']::public.custom_id_type[], description = NULL
WHERE id_types = ARRAY['Other']::public.custom_id_type[]
    AND lower(description) IN ('sp', 'startpos');

UPDATE public.level_custom_copies
SET id_types = array_append(id_types, 'StartPos'::public.custom_id_type)
WHERE id_types <> ARRAY['Other']::public.custom_id_type[]
    AND NOT ('StartPos'::public.custom_id_type = ANY(id_types))
    AND (description ILIKE '%sp%' OR description ILIKE '%startpos%');
