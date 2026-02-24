CREATE OR REPLACE VIEW aredl.submissions_with_priority AS
SELECT 
    *,
    (EXTRACT(EPOCH FROM CLOCK_TIMESTAMP()) - EXTRACT(EPOCH FROM created_at))::BIGINT + 
    CASE WHEN priority = TRUE THEN 21600 ELSE 0 END AS priority_value
FROM aredl.submissions;

CREATE OR REPLACE VIEW arepl.submissions_with_priority AS
SELECT 
    *,
    (EXTRACT(EPOCH FROM CLOCK_TIMESTAMP()) - EXTRACT(EPOCH FROM created_at))::BIGINT + 
    CASE WHEN priority = TRUE THEN 21600 ELSE 0 END AS priority_value
FROM arepl.submissions;