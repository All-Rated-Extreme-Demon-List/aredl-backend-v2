DROP INDEX public.records_level_completion_idx;
ALTER TABLE public.records DROP COLUMN completion_time_verified;
CREATE INDEX records_level_completion_idx ON public.records
    (list_id, level_id, completion_time, id);
