-- USERS table
CREATE TABLE users (
    user_id TEXT NOT NULL PRIMARY KEY,
    -- UUID
    --
    username TEXT UNIQUE,
    email TEXT UNIQUE,
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

CREATE INDEX idx_users_deleted ON users (deleted_at);

-- CHARACTER table
CREATE TABLE characters (
    character_id TEXT NOT NULL PRIMARY KEY,
    -- UUID
    -- 
    user_id TEXT NOT NULL,
    -- 
    character_name TEXT NOT NULL,
    portrait TEXT NOT NULL,
    -- 
    max_area_level INT NOT NULL DEFAULT 0,
    resource_gems FLOAT NOT NULL DEFAULT 0,
    resource_shards FLOAT NOT NULL DEFAULT 0,
    -- 
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    deleted_at TIMESTAMP DEFAULT NULL,
    --
    FOREIGN KEY(user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

CREATE INDEX idx_characters_user_id ON characters (user_id, deleted_at);

CREATE INDEX idx_characters_deleted ON characters (deleted_at);

-- CHARACTER_DATA table
-- contain serialized data
CREATE TABLE characters_data (
    character_id TEXT NOT NULL PRIMARY KEY,
    -- UUID
    -- 
    data_version TEXT NOT NULL,
    inventory_data BLOB NOT NULL,
    --
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    --
    FOREIGN KEY(character_id) REFERENCES characters(character_id) ON DELETE CASCADE
);

-- CHARACTER_AREA_COMPLETED table
CREATE TABLE character_area_completed (
    character_id TEXT NOT NULL,
    area_id TEXT NOT NULL,
    --
    max_area_level INT NOT NULL DEFAULT 0,
    --
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    --
    PRIMARY KEY(character_id, area_id),
    FOREIGN KEY(character_id) REFERENCES characters(character_id) ON DELETE CASCADE
);

CREATE INDEX idx_character_area_completed_character ON character_area_completed (character_id);

-- GAME_SESSIONS table
DROP TABLE game_sessions;

CREATE TABLE game_sessions (
    session_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    -- 
    character_id TEXT NOT NULL,
    -- 
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    ended_at TIMESTAMP NOT NULL DEFAULT '9999-1-1 23:59:59',
    -- 
    UNIQUE(character_id, ended_at),
    FOREIGN KEY(character_id) REFERENCES characters(character_id) ON DELETE CASCADE
);

CREATE INDEX idx_session_character_id ON game_sessions (character_id, ended_at);

-- SAVED_GAME_INSTANCES table
DROP TABLE saved_game_instances;

CREATE TABLE saved_game_instances (
    character_id TEXT NOT NULL PRIMARY KEY,
    -- 
    area_id TEXT NOT NULL,
    area_level INT NOT NULL,
    -- 
    saved_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    -- 
    data_version TEXT NOT NULL,
    game_data BLOB NOT NULL,
    --
    FOREIGN KEY(character_id) REFERENCES characters(character_id) ON DELETE CASCADE
);

-- LEADERBOARD table 
-- replaced by looking into characters and completed_areas tables
DROP TABLE leaderboard;

-- USER_ACTIVITY_LOG table 
CREATE TABLE user_activity_log (
    log_id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    --
    user_id TEXT NOT NULL,
    --
    activity_type TEXT NOT NULL,
    details TEXT,
    --
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    --
    FOREIGN KEY(user_id) REFERENCES users(user_id) ON DELETE CASCADE
);