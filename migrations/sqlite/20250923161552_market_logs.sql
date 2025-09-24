ALTER TABLE market
ADD COLUMN item_crit_chance REAL;

ALTER TABLE market
ADD COLUMN item_crit_damage REAL;

ALTER TABLE market
ADD COLUMN deleted_by TEXT;

-- ALTER TABLE market ADD CONSTRAINT fk_deleted_by FOREIGN KEY (deleted_by) REFERENCES characters (character_id) ON DELETE SET NULL;