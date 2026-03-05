import pandas as pd

TIER_WEIGHTS = [1000, 1000, 1000, 1000, 1000, 900, 800, 700, 600, 500]
# TIER_WEIGHTS = [1000] * 10
TIER_LEVELS = [1, 50, 100, 150, 200, 260, 330, 410, 500, 600]

AREA_LEVELS = [1, 50, 100, 150, 200, 260, 330, 410, 500, 600, 700, 800, 900, 1000, 1500]


def tweak_weight(area_level: int, tier_level: int, tier_weight: int) -> float:
    if area_level < tier_level:
        return 0.0
    delta = area_level - tier_level
    factor = 1.0 + delta * tier_level / 10_000.0
    return tier_weight * factor


def compute_roll_chance(area_level: int):
    tweaked_weights = [
        tweak_weight(area_level, tier_level, tier_weight)
        for tier_level, tier_weight in zip(TIER_LEVELS, TIER_WEIGHTS)
    ]
    total_weight = sum(tweaked_weights)
    return [w / total_weight if total_weight > 0 else 0 for w in tweaked_weights]


rows = []
for area_level in AREA_LEVELS:
    rows.append(compute_roll_chance(area_level))

df = pd.DataFrame(
    rows,
    index=[f"Area {lvl}" for lvl in AREA_LEVELS],
    columns=[f"T{lvl}" for lvl in TIER_LEVELS],
)

# convert to percentage strings
df_percent = df.map(lambda x: f"{x:.2%}")

print(df_percent)
