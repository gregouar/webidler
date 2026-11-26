DROP INDEX IF EXISTS idx_market_name;

CREATE INDEX idx_market_name ON market (realm_id, item_name)
WHERE
    deleted_at IS NULL;
