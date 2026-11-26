CREATE TABLE
    stash_items_affixes (
        stash_item_id BIGINT NOT NULL,
        base_item_id TEXT,
        affix_group_id TEXT NOT NULL,
        affix_level INTEGER NOT NULL,
        roll_value INTEGER NOT NULL,
        realm_id TEXT NOT NULL,
        deleted_at TIMESTAMPTZ,
        FOREIGN KEY (stash_item_id) REFERENCES stash_items (stash_item_id) ON DELETE CASCADE
    );

CREATE INDEX idx_stash_items_affixes_item_id ON stash_items_affixes (stash_item_id);

CREATE INDEX idx_stash_items_affixes_filter ON stash_items_affixes (
    realm_id,
    base_item_id,
    affix_group_id,
    affix_level,
    stash_item_id
)
WHERE
    deleted_at IS NULL;