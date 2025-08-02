
CREATE TABLE saved_game_instances (
    user_id TEXT NOT NULL PRIMARY KEY ,
    saved_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    game_data BLOB NOT NULL
);
