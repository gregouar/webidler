ALTER TABLE
    characters
ADD
    COLUMN played_time_seconds REAL NOT NULL DEFAULT 0;

UPDATE
    characters
SET
    played_time_seconds = COALESCE(
        (
            SELECT
                SUM(gs.elapsed_time)
            FROM
                game_stats gs
            WHERE
                gs.character_id = characters.character_id
        ),
        0
    );

UPDATE
    characters
SET
    played_time_seconds = 0
WHERE
    played_time_seconds IS NULL;
