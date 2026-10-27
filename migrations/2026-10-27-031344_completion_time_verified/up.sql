ALTER TABLE public.records
    ADD COLUMN completion_time_verified BOOLEAN NOT NULL DEFAULT TRUE;

DROP INDEX public.records_level_completion_idx;
CREATE INDEX records_level_completion_idx ON public.records
    (list_id, level_id, completion_time_verified DESC, completion_time, id);
