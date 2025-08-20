DROP TABLE users;

CREATE TABLE users (
    user_id TEXT NOT NULL PRIMARY KEY,
    -- UUID
    --
    username TEXT UNIQUE,
    email_crypt BLOB,
    email_hash BLOB UNIQUE,
    password_hash TEXT,
    --
    terms_accepted_at TIMESTAMP NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    --
    max_characters INT NOT NULL,
    --
    last_login_at TIMESTAMP DEFAULT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL
);

CREATE INDEX idx_users_username ON users (username);

CREATE INDEX idx_users_email ON users (email_hash);

CREATE INDEX idx_users_deleted ON users (deleted_at);