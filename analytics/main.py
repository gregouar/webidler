import math

import matplotlib.pyplot as plt
import numpy as np

max_level = 1200
factor_gold = 0.12
factor_price = 0.31

levels = np.arange(1, max_level + 1)


def simulate(damage_multiplier: float, factor_life: float) -> list[int]:
    kills_needed = []

    gold = 0.0
    damage = 1.0
    skill_level = 1
    next_cost = 1.0
    kills = 0

    for level in levels:
        monster_life = 10 ** ((level - 1) * factor_life)
        gold_reward = 10 ** ((level - 1) * factor_gold)

        while damage < monster_life:

            # If we can't afford upgrade, jump directly
            if gold < next_cost:
                required_gold = next_cost - gold
                kills_to_afford = math.ceil(required_gold / gold_reward)

                gold += kills_to_afford * gold_reward
                kills += kills_to_afford

            # Buy upgrade
            gold -= next_cost
            next_cost += 10 ** ((skill_level - 1) * factor_price)
            damage *= damage_multiplier
            skill_level += 1

        kills_needed.append(math.log10(kills + 1))

    print(
        damage_multiplier, ": ", kills_needed[max_level // 2], " - ", kills_needed[-1]
    )

    return kills_needed


plt.figure()
plt.plot(levels, simulate(1.5, 0.12))
plt.plot(levels, simulate(1.3, 0.07765))
plt.legend(["1.5", "1.3"])
plt.xlabel("Level")
plt.ylabel("Time")
plt.title("Time to Reach Damage = Monster Life")
plt.show()
