ALTER TABLE oauth_requests
ADD COLUMN nonce VARCHAR;
UPDATE oauth_requests SET nonce = '' WHERE nonce IS NULL;
ALTER TABLE oauth_requests
ALTER COLUMN nonce SET NOT NULL;