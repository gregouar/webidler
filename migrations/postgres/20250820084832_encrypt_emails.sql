ALTER TABLE
    users DROP CONSTRAINT users_email_key;

ALTER TABLE
    users DROP COLUMN email;

ALTER TABLE
    users
ADD
    COLUMN email_crypt BYTEA,
ADD
    COLUMN email_hash BYTEA UNIQUE;

CREATE INDEX idx_users_email ON users (email_hash);