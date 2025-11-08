CREATE TABLE arepl.shifts (
  LIKE public.shifts INCLUDING ALL
);
CREATE TABLE arepl.recurrent_shifts (
  LIKE public.recurrent_shifts INCLUDING ALL
);

ALTER TABLE public.shifts           SET SCHEMA aredl;
ALTER TABLE public.recurrent_shifts SET SCHEMA aredl;