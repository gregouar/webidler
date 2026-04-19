ALTER TABLE
    characters
ADD
    COLUMN played_time_seconds DOUBLE PRECISION DEFAULT 0;

UPDATE
    characters c
SET
    played_time_seconds = stats.total_elapsed_time
FROM
    (
        SELECT
            character_id,
            COALESCE(SUM(elapsed_time), 0) AS total_elapsed_time
        FROM
            game_stats
        GROUP BY
            character_id
    ) AS stats
WHERE
    c.character_id = stats.character_id;

UPDATE
    characters
SET
    played_time_seconds = 0
WHERE
    played_time_seconds IS NULL;

ALTER TABLE
    characters
ALTER COLUMN
    played_time_seconds
SET
    NOT NULL;