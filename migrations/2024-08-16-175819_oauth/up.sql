CREATE TABLE oauth_requests (
    csrf_state VARCHAR NOT NULL,
    pkce_verifier VARCHAR NOT NULL,
    nonce VARCHAR NOT NULL,
    use_message BOOL NOT NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY(csrf_state)
);