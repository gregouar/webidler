CREATE TABLE users (
    user_id TEXT PRIMARY KEY,
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    terms_accepted_at TEXT NOT NULL,
    is_admin BIT NOT NULL,
    is_deleted BIT NOT NULL,
    
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_users_username ON leaderboard (area_level, time_played_seconds, created_at);
