ALTER TABLE
    market
ADD
    COLUMN item_crit_chance REAL,
    COLUMN item_crit_damage REAL,
    COLUMN deleted_by TEXT,
    FOREIGN KEY(deleted_by) REFERENCES characters(character_id) ON DELETE
    SET
        NULL;