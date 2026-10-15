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
    ADD CONSTRAINT level_custom_copies_copy_id_key UNIQUE (copy_id);

ALTER TYPE public.custom_id_type ADD VALUE 'Startpos';
