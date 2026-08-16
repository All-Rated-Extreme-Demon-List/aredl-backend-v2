CREATE TYPE oauth_provider AS ENUM ('Discord', 'Patreon', 'Google', 'Twitch');

ALTER TABLE oauth_requests
	ADD COLUMN provider oauth_provider NOT NULL DEFAULT 'Discord',
	ADD COLUMN user_id uuid REFERENCES users(id) ON DELETE CASCADE,
	ALTER COLUMN pkce_verifier DROP NOT NULL;

CREATE TABLE oauth_connected_accounts (
	id uuid DEFAULT uuid_generate_v4(),
	user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
	provider oauth_provider NOT NULL,
	provider_user_id TEXT NOT NULL,
	provider_user_name TEXT,
	created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY(id),
	UNIQUE(provider, user_id),
	UNIQUE(provider, provider_user_id)
);

CREATE TABLE oauth_tokens (
	provider oauth_provider NOT NULL,
	access_token TEXT,
	refresh_token TEXT,
	expires_at TIMESTAMPTZ,
	updated_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
	PRIMARY KEY(provider)
);
