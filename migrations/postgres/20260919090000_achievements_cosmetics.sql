CREATE TABLE user_achievements (
    user_id UUID NOT NULL,
    achievement_id TEXT NOT NULL,
    unlocked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, achievement_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

CREATE TABLE user_cosmetics (
    user_id UUID NOT NULL,
    cosmetic_id TEXT NOT NULL,
    unlocked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, cosmetic_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

CREATE TABLE user_pets (
    user_id UUID NOT NULL,
    pet_id TEXT NOT NULL,
    unlocked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, pet_id),
    FOREIGN KEY (user_id) REFERENCES users(user_id) ON DELETE CASCADE
);

ALTER TABLE characters ADD COLUMN cosmetic_title TEXT;
ALTER TABLE characters ADD COLUMN cosmetic_badge TEXT;
ALTER TABLE characters_data ADD COLUMN pets_data BYTEA;

UPDATE users
SET chat_badge = CASE chat_badge
    WHEN 'Developer' THEN 'badge_dev'
    WHEN 'WitchHunter' THEN 'badge_witch'
    WHEN 'CrucibleChaosGold' THEN 'badge_chaos_gold'
    WHEN 'CrucibleChaosSilver' THEN 'badge_chaos_silver'
    WHEN 'CrucibleChaosBronze' THEN 'badge_chaos_bronze'
    ELSE chat_badge
END;

UPDATE characters
SET portrait = CASE portrait
    WHEN 'adventurers/human_male_1_gdca.webp' THEN 'human_male_1'
    ELSE substring(portrait FROM 13 FOR char_length(portrait) - 17)
END
WHERE portrait LIKE 'adventurers/%.webp';
