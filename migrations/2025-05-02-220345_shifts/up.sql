CREATE TYPE shift_status AS ENUM (
  'Running',
  'Completed',
  'Expired'
);

CREATE TABLE aredl_shifts (
  id               UUID           PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id          UUID           NOT NULL REFERENCES users(id),
  target_count     INT            NOT NULL,
  completed_count  INT            NOT NULL DEFAULT 0,
  start_at         TIMESTAMPTZ    NOT NULL,
  end_at           TIMESTAMPTZ    NOT NULL,
  status           shift_status   NOT NULL DEFAULT 'Running',
  created_at       TIMESTAMPTZ    NOT NULL DEFAULT NOW(),
  updated_at       TIMESTAMPTZ    NOT NULL DEFAULT NOW()
);

CREATE INDEX ix_aredl_shifts_user_status   ON aredl_shifts(user_id, status);
CREATE INDEX ix_aredl_shifts_start         ON aredl_shifts(start_at);
CREATE INDEX ix_aredl_shifts_end           ON aredl_shifts(end_at);

CREATE TYPE weekday AS ENUM (
  'Monday','Tuesday','Wednesday','Thursday','Friday','Saturday','Sunday'
);

CREATE TABLE aredl_recurrent_shifts (
  id             UUID           PRIMARY KEY DEFAULT gen_random_uuid(),
  user_id        UUID           NOT NULL REFERENCES users(id),
  weekday        weekday        NOT NULL,
  start_hour     INT           NOT NULL,
  duration       INT           NOT NULL,
  target_count   INT            NOT NULL,
  created_at     TIMESTAMPTZ    NOT NULL DEFAULT NOW(),
  updated_at     TIMESTAMPTZ    NOT NULL DEFAULT NOW()
);

CREATE INDEX ix_aredl_recurrent_shifts_user_weekday
  ON aredl_recurrent_shifts(user_id, weekday);
