CREATE TABLE oauth_requests (
    csrf_state VARCHAR NOT NULL,
    pkce_verifier VARCHAR NOT NULL,
    nonce VARCHAR NOT NULL,
	callback VARCHAR,
    created_at TIMESTAMPTZ DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(csrf_state)
);