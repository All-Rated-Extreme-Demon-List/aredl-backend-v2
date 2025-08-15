

CREATE OR REPLACE FUNCTION aredl.update_record_time()
RETURNS TRIGGER AS $$
BEGIN
  IF (to_jsonb(NEW) - 'updated_at' - 'placement_order')
       IS DISTINCT FROM
     (to_jsonb(OLD) - 'updated_at' - 'placement_order')
  THEN
    NEW.updated_at := now();
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION arepl.update_record_time()
RETURNS TRIGGER AS $$
BEGIN
  IF (to_jsonb(NEW) - 'updated_at' - 'placement_order')
       IS DISTINCT FROM
     (to_jsonb(OLD) - 'updated_at' - 'placement_order')
  THEN
    NEW.updated_at := now();
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

UPDATE aredl.records
SET updated_at = created_at
WHERE updated_at IS DISTINCT FROM created_at;

UPDATE arepl.records
SET updated_at = created_at
WHERE updated_at IS DISTINCT FROM created_at;
