CREATE TABLE matview_refresh_log (
  view_name   TEXT        PRIMARY KEY,
  last_refresh TIMESTAMPTZ NOT NULL
);