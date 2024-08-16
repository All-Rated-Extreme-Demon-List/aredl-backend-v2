CREATE TABLE oauth_requests (
    csrf_state VARCHAR NOT NULL,
    pkce_verifier VARCHAR NOT NULL,
    nonce VARCHAR NOT NULL,
	opener_origin VARCHAR,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(csrf_state)
);