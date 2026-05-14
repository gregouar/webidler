ALTER TABLE
    stashes
ADD
    COLUMN owner_id TEXT NOT NULL DEFAULT '';

UPDATE
    stashes
SET
    owner_id = user_id
WHERE
    owner_id = '';

CREATE INDEX idx_stashes_owner_id ON stashes (realm_id, owner_id);
