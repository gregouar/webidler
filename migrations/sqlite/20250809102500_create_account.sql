-- USERS table
CREATE TABLE users (
    user_id TEXT PRIMARY KEY,
    -- 
    username TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    -- 
    terms_accepted_at TIMESTAMP NOT NULL,
    is_admin BIT NOT NULL,
    is_deleted BIT NOT NULL,
    -- 
    max_characters INT NOT NULL,
    -- 
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
);

CREATE INDEX idx_users_username ON leaderboard (area_level, time_played_seconds, created_at);

-- CHARACTER table
-- inventory/ascended passives are stored in separate table CHARACTERS_DATA that will be added later 
CREATE TABLE characters (
    character_id TEXT PRIMARY KEY,
    -- 
    FOREIGN KEY(user_id) REFERENCES users(user_id) NOT NULL,
    -- 
    character_name TEXT,
    portrait TEXT,
    -- 
    max_area_level INT NOT NULL DEFAULT 0,
    resource_gems FLOAT NOT NULL DEFAULT 0,
    resource_shards FLOAT NOT NULL DEFAULT 0,
    -- 
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
);

CREATE TABLE character_area_completed (
    FOREIGN KEY(character_id) REFERENCES characters(character_id) NOT NULL PRIMARY KEY,
    area_id TEXT NOT NULL PRIMARY KEY,
    --
    max_area_level INT NOT NULL DEFAULT 0,
);

-- GAME_SESSIONS table
DROP TABLE game_sessions;

CREATE TABLE game_sessions (
    session_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    -- 
    FOREIGN KEY(character_id) REFERENCES characters(character_id) NOT NULL,
    -- 
    area_id TEXT NOT NULL,
    -- 
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP DEFAULT NULL,
    -- 
    UNIQUE(character_id, ended_at)
);

CREATE INDEX idx_session_user_id ON game_sessions (user_id, ended_at);

-- SAVED_GAME_INSTANCES table
DROP TABLE CREATE TABLE saved_game_instances;

CREATE TABLE saved_game_instances (
    FOREIGN KEY(character_id) REFERENCES characters(character_id) NOT NULL PRIMARY KEY,
    -- 
    saved_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 
    game_data BLOB NOT NULL
);

-- LEADERBOARD table (will get from characters tables directly now)
DROP TABLE leaderboard;