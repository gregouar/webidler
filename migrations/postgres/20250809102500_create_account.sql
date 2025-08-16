-- USERS table
CREATE TABLE users (
    user_id UUID NOT NULL PRIMARY KEY,
    username TEXT UNIQUE,
    email TEXT UNIQUE,
    password_hash TEXT,
    terms_accepted_at TIMESTAMPTZ NOT NULL,
    is_admin BOOLEAN NOT NULL DEFAULT FALSE,
    max_characters SMALLINT3 NOT NULL,
    last_login_at TIMESTAMPTZ DEFAULT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ DEFAULT NULL
);

CREATE INDEX idx_users_username ON users (username);

CREATE INDEX idx_users_deleted ON users (deleted_at);

-- CHARACTERS table
CREATE TABLE characters (
    character_id UUID NOT NULL PRIMARY KEY,
    user_id UUID NOT NULL,
    character_name TEXT NOT NULL,
    portrait TEXT NOT NULL,
    max_area_level INTEGER NOT NULL DEFAULT 0,
    resource_gems DOUBLE PRECISION NOT NULL DEFAULT 0,
    resource_shards DOUBLE PRECISION NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    deleted_at TIMESTAMPTZ DEFAULT NULL,
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

CREATE INDEX idx_characters_user_id ON characters (user_id, deleted_at);

CREATE INDEX idx_characters_deleted ON characters (deleted_at);

-- CHARACTERS_DATA table
CREATE TABLE characters_data (
    character_id UUID NOT NULL PRIMARY KEY,
    data_version TEXT NOT NULL,
    inventory_data BYTEA NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (character_id) REFERENCES characters(character_id) ON DELETE CASCADE
);

-- CHARACTER_AREA_COMPLETED table
CREATE TABLE character_area_completed (
    character_id UUID NOT NULL,
    area_id UUID NOT NULL,
    max_area_level INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (character_id, area_id),
    FOREIGN KEY (character_id) REFERENCES characters(character_id) ON DELETE CASCADE
);

CREATE INDEX idx_character_area_completed_character ON character_area_completed (character_id);

-- GAME_SESSIONS table
DROP TABLE IF EXISTS game_sessions;

CREATE TABLE game_sessions (
    session_id BIGSERIAL PRIMARY KEY,
    character_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    ended_at TIMESTAMPTZ NOT NULL DEFAULT '9999-01-01 23:59:59',
    UNIQUE (character_id, ended_at),
    FOREIGN KEY (character_id) REFERENCES characters(character_id) ON DELETE CASCADE
);

CREATE INDEX idx_session_character_id ON game_sessions (character_id, ended_at);

-- SAVED_GAME_INSTANCES table
DROP TABLE IF EXISTS saved_game_instances;

CREATE TABLE saved_game_instances (
    character_id UUID NOT NULL PRIMARY KEY,
    area_id UUID NOT NULL,
    area_level INTEGER NOT NULL,
    saved_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    data_version TEXT NOT NULL,
    game_data BYTEA NOT NULL,
    FOREIGN KEY (character_id) REFERENCES characters(character_id) ON DELETE CASCADE
);

-- LEADERBOARD table (removed)
DROP TABLE IF EXISTS leaderboard;

-- USER_ACTIVITY_LOG table
CREATE TABLE user_activity_log (
    log_id BIGSERIAL PRIMARY KEY,
    user_id UUID NOT NULL,
    activity_type TEXT NOT NULL,
    details TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);