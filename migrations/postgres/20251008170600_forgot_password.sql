CREATE TABLE password_reset_tokens (
    token_id BIGSERIAL NOT NULL PRIMARY KEY,
    user_id UUID NOT NULL,
    token_hash BYTEA NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

CREATE INDEX idx_password_reset_token ON password_reset_tokens(user_id, token_hash, expires_at)
WHERE
    used_at IS NULL;

CREATE INDEX idx_password_reset_user_id ON password_reset_tokens(user_id, expires_at);