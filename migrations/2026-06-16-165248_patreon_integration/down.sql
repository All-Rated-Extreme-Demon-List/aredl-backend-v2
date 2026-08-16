DROP TABLE oauth_tokens;
DROP TABLE oauth_connected_accounts;

ALTER TABLE oauth_requests
	DROP COLUMN user_id,
	DROP COLUMN provider,
	ALTER COLUMN pkce_verifier SET NOT NULL;
	
DROP TYPE oauth_provider;
