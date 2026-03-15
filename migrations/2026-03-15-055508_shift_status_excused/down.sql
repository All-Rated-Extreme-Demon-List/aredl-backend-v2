UPDATE aredl_shifts
SET status = 'Expired'
WHERE status = 'Excused';

ALTER TYPE shift_status RENAME TO shift_status_old;

CREATE TYPE shift_status AS ENUM (
  'Running',
  'Completed',
  'Expired'
);

ALTER TABLE aredl_shifts
ALTER COLUMN status TYPE shift_status
USING status::text::shift_status;

DROP TYPE shift_status_old;
