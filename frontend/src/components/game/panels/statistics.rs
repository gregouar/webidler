use std::collections::{BTreeMap, BTreeSet};

use leptos::{html::*, prelude::*};

use shared::data::{
    chance::BoundedChance,
    character_status::StatusId,
    skill::{DamageType, RestoreType, SkillEffect, SkillEffectType, SkillType},
    stat_effect::{EffectsMap, StatSkillFilter, StatType},
    trigger::TriggerSpecs,
};
use strum::IntoEnumIterator;

use crate::components::{
    data_context::DataContext,
    game::GameContext,
    shared::tooltips::{
        effects_tooltip::{self, format_multiplier_stat_name},
        skill_tooltip::{skill_filter_str, skill_type_str},
        trigger_tooltip,
    },
    ui::{
        Separator,
        card::{CardHeader, CardInset, CardInsetTitle, MenuCard},
        menu_panel::MenuPanel,
        number::{Number, format_duration, format_number},
    },
    utils::stats_computations::{
        compute_stats_effects_crit_chance_value, compute_stats_effects_crit_damage_value,
        compute_stats_effects_damage_value, compute_stats_effects_mana_cost_value,
        compute_stats_effects_restore_value, compute_stats_effects_speed_value,
        compute_stats_effects_status_duration_value, compute_stats_effects_status_power_value,
        compute_stats_effects_status_value, compute_stats_effects_success_chance_value,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct HitDamageKind {
    skill_type: SkillType,
    damage_type: DamageType,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum DamageMultiplierStatKind {
    Hit(HitDamageKind),
    Status {
        skill_type: SkillType,
        status_id: StatusId,
    },
}

#[derive(Debug, Clone, PartialEq)]
struct DamageMultiplierStat {
    kind: DamageMultiplierStatKind,
    label: String,
    multiplier: f64,
}

#[component]
pub fn StatisticsPanel(open: RwSignal<bool>) -> impl IntoView {
    let game_context = expect_context::<GameContext>();
    let data_context = expect_context::<DataContext>();
    // let effect = move |stat: StatType, modifier: Modifier| {
    //     game_context
    //         .player_specs
    //         .read()
    //         .character_specs
    //         .effects
    //         .0
    //         .get(&(stat, modifier, false))
    //         .copied()
    //         .unwrap_or_default()
    // };

    let effects_map = Memo::new(move |_| {
        EffectsMap::from(
            game_context
                .player_specs
                .read()
                .character_specs
                .effects
                .clone(),
        )
    });

    let damage_multipliers = Memo::new(move |_| {
        let player_specs = game_context.player_specs.read();
        let statuses_specs = data_context.statuses_specs.read();
        let mut kinds = BTreeSet::new();
        let mut status_multipliers = BTreeMap::new();

        {
            let mut add_damage_effect =
                |source_id: &String,
                 skill_type: SkillType,
                 effect: &SkillEffect,
                 allow_zero_status: bool| match &effect.effect_type {
                    SkillEffectType::FlatDamage { damage, .. } => {
                        for &damage_type in damage
                            .iter()
                            .filter(|(_, value)| value.min.get() > 0.0 || value.max.get() > 0.0)
                            .map(|(damage_type, _)| damage_type)
                        {
                            kinds.insert(HitDamageKind {
                                skill_type,
                                damage_type,
                            });
                        }
                    }
                    SkillEffectType::ApplyStatus {
                        status_id, value, ..
                    } if value.min.get() > 0.0 || value.max.get() > 0.0 || allow_zero_status => {
                        if let Some(status_specs) = statuses_specs.get(status_id)
                            && let Some(damage_type) = status_specs.damage_type
                        {
                            status_multipliers
                                .entry((skill_type, status_id.clone()))
                                .or_insert_with(|| {
                                    compute_stats_effects_status_value(
                                        &effects_map.read(),
                                        &effect.ignore_stat_effects,
                                        Some(source_id),
                                        Some(skill_type),
                                        status_id,
                                        Some(damage_type),
                                        status_specs.debuff,
                                    )
                                });
                        }
                    }
                    _ => {}
                };

            for skill in player_specs
                .character_specs
                .skills_specs
                .iter()
                .filter(|skill| skill.usable)
            {
                for effect in skill
                    .targets
                    .iter()
                    .flat_map(|target| target.effects.iter())
                {
                    add_damage_effect(&skill.skill_id, skill.skill_type, effect, false);
                }
            }

            for trigger_effect in player_specs
                .character_specs
                .triggers
                .iter()
                .flat_map(|(_, owned_triggers)| owned_triggers.iter())
                .map(|owned_trigger| &owned_trigger.trigger_effect)
            {
                for effect in &trigger_effect.effects {
                    add_damage_effect(
                        &trigger_effect.trigger_id,
                        trigger_effect.skill_type,
                        effect,
                        !trigger_effect.modifiers.is_empty(),
                    );
                }
            }
        }

        let hit_stats = kinds.into_iter().map(|kind| DamageMultiplierStat {
            kind: DamageMultiplierStatKind::Hit(kind),
            label: format_hit_damage_kind(kind),
            multiplier: compute_stats_effects_damage_value(
                &effects_map.read(),
                kind.skill_type,
                kind.damage_type,
                true,
            ),
        });
        let status_stats =
            status_multipliers
                .into_iter()
                .map(
                    |((skill_type, status_id), multiplier)| DamageMultiplierStat {
                        label: format!(
                            "{} Damage{}",
                            statuses_specs
                                .get(&status_id)
                                .map(|status| status.name.as_str())
                                .unwrap_or(status_id.as_str()),
                            skill_filter_str(
                                &StatSkillFilter {
                                    skill_type: Some(skill_type),
                                    ..Default::default()
                                },
                                " with ",
                                true,
                            ),
                        ),
                        kind: DamageMultiplierStatKind::Status {
                            skill_type,
                            status_id,
                        },
                        multiplier,
                    },
                );

        hit_stats.chain(status_stats).collect::<Vec<_>>()
    });

    view! {
        <MenuPanel open=open center=false>
            <Show when=move || open.get()>
                <MenuCard class="h-full min-h-0 overflow-hidden">
                    <CardHeader title="Statistics" on_close=move || open.set(false) />

                    <div class="grid min-h-0 flex-1 grid-rows-3 gap-2 xl:gap-4 overflow-hidden">
                        <div class="row-span-2 min-h-0 grid grid-cols-2 xl:grid-cols-3 gap-2 xl:gap-4 overflow-y-auto">

                            <StatCategory title="Game">
                                <Stat
                                    label="Elapsed Time"
                                    value=move || {
                                        format_duration(
                                            game_context.game_stats.read().elapsed_time,
                                            true,
                                        )
                                    }
                                />
                                <Stat
                                    label="Areas Completed"
                                    value=move || {
                                        game_context.game_stats.read().areas_completed.to_string()
                                    }
                                />
                                <Stat
                                    label="Monsters Killed"
                                    value=move || {
                                        game_context.game_stats.read().monsters_killed.to_string()
                                    }
                                />
                                <Stat
                                    label="Player Deaths"
                                    value=move || {
                                        game_context.game_stats.read().player_deaths.to_string()
                                    }
                                />
                                <Stat
                                    label="Highest Area Level (this grind)"
                                    value=move || {
                                        game_context.area_state.read().max_area_level.to_string()
                                    }
                                />
                                <Stat
                                    label="Highest Area Level (ever)"
                                    value=move || {
                                        game_context
                                            .area_state
                                            .read()
                                            .max_area_level_ever
                                            .to_string()
                                    }
                                />
                                <Stat
                                    label="Power Shards Unlocked up to Level"
                                    value=move || {
                                        game_context
                                            .area_state
                                            .read()
                                            .max_power_shard_level_ever
                                            .to_string()
                                    }
                                />
                                <Stat
                                    label="Average Damage per second"
                                    value=move || {
                                        format_number(game_context.game_local_stats.average_dps())
                                    }
                                />
                                <Stat
                                    label="Average Damage per Hit"
                                    value=move || {
                                        format_number(
                                            game_context.game_local_stats.average_damage_tick(),
                                        )
                                    }
                                />
                            </StatCategory>

                            <StatCategory title="Character">
                                <Stat
                                    label="Name"
                                    value=move || {
                                        game_context
                                            .player_base_specs
                                            .read()
                                            .character_static
                                            .name
                                            .clone()
                                    }
                                />
                                <Stat
                                    label="Level"
                                    value=move || {
                                        game_context.player_base_specs.read().level.to_string()
                                    }
                                />
                                <Stat
                                    label="Maximum Life"
                                    value=move || {
                                        format_number(
                                            game_context
                                                .player_specs
                                                .read()
                                                .character_specs
                                                .character_attrs
                                                .max_life
                                                .get(),
                                        )
                                    }
                                />
                                <Stat
                                    label="Life Regeneration per second"
                                    value=move || {
                                        let value = *game_context
                                            .player_specs
                                            .read()
                                            .character_specs
                                            .character_attrs
                                            .life_regen * 0.1;
                                        if value == 0.0 {
                                            "-".into()
                                        } else {
                                            format!("{:.1}%", value)
                                        }
                                    }
                                />
                                <Stat
                                    label="Maximum Mana"
                                    value=move || {
                                        format_number(
                                            game_context
                                                .player_specs
                                                .read()
                                                .character_specs
                                                .character_attrs
                                                .max_mana
                                                .get(),
                                        )
                                    }
                                />
                                <Stat
                                    label="Mana Regeneration per second"
                                    value=move || {
                                        let value = *game_context
                                            .player_specs
                                            .read()
                                            .character_specs
                                            .character_attrs
                                            .mana_regen * 0.1;
                                        if value == 0.0 {
                                            "-".into()
                                        } else {
                                            format!("{:.1}%", value)
                                        }
                                    }
                                />
                                <Stat
                                    label="Gold Find"
                                    value=move || {
                                        format!(
                                            "{}%",
                                            format_number(
                                                game_context.player_specs.read().gold_find.get(),
                                            ),
                                        )
                                    }
                                />
                                <Stat
                                    label="Movement Cooldown"
                                    value=move || {
                                        format!(
                                            "{:.2}s",
                                            game_context.player_specs.read().movement_cooldown.get(),
                                        )
                                    }
                                />
                            </StatCategory>

                            <StatCategory title="Defense">
                                <Stat
                                    label="Physical Defense"
                                    value=move || {
                                        format!(
                                            "{:.0}",
                                            *game_context
                                                .player_specs
                                                .read()
                                                .character_specs
                                                .character_attrs
                                                .armor
                                                .get(&DamageType::Physical)
                                                .cloned()
                                                .unwrap_or_default(),
                                        )
                                    }
                                />
                                <Stat
                                    label="Fire Defense"
                                    value=move || {
                                        format!(
                                            "{:.0}",
                                            *game_context
                                                .player_specs
                                                .read()
                                                .character_specs
                                                .character_attrs
                                                .armor
                                                .get(&DamageType::Fire)
                                                .cloned()
                                                .unwrap_or_default(),
                                        )
                                    }
                                />
                                <Stat
                                    label="Poison Defense"
                                    value=move || {
                                        format!(
                                            "{:.0}",
                                            *game_context
                                                .player_specs
                                                .read()
                                                .character_specs
                                                .character_attrs
                                                .armor
                                                .get(&DamageType::Poison)
                                                .cloned()
                                                .unwrap_or_default(),
                                        )
                                    }
                                />
                                <Stat
                                    label="Storm Defense"
                                    value=move || {
                                        format!(
                                            "{:.0}",
                                            *game_context
                                                .player_specs
                                                .read()
                                                .character_specs
                                                .character_attrs
                                                .armor
                                                .get(&DamageType::Storm)
                                                .cloned()
                                                .unwrap_or_default(),
                                        )
                                    }
                                />
                                {move || {
                                    let block_spell = game_context
                                        .player_specs
                                        .read()
                                        .character_specs
                                        .character_attrs
                                        .block
                                        .get(&SkillType::Attack)
                                        .copied()
                                        .unwrap_or_default();
                                    (block_spell.value.get() != 0.0)
                                        .then(move || {
                                            view! {
                                                <Stat
                                                    label="Attack Block Chance (max 80%)"
                                                    value=move || { format_chance(&block_spell) }
                                                />
                                            }
                                        })
                                }}
                                {move || {
                                    let block_spell = game_context
                                        .player_specs
                                        .read()
                                        .character_specs
                                        .character_attrs
                                        .block
                                        .get(&SkillType::Spell)
                                        .copied()
                                        .unwrap_or_default();
                                    (block_spell.value.get() != 0.0)
                                        .then(move || {
                                            view! {
                                                <Stat
                                                    label="Spell Block Chance (max 80%)"
                                                    value=move || { format_chance(&block_spell) }
                                                />
                                            }
                                        })
                                }}
                                {move || {
                                    let block_damage = game_context
                                        .player_specs
                                        .read()
                                        .character_specs
                                        .character_attrs
                                        .block_damage
                                        .get() as f64;
                                    (block_damage != 0.0)
                                        .then(move || {
                                            view! {
                                                <Stat
                                                    label="Blocked Damage Taken"
                                                    value=move || format!("{:.0}%", block_damage)
                                                />
                                            }
                                        })
                                }}
                                {move || {
                                    let evade_damage = game_context
                                        .player_specs
                                        .read()
                                        .character_specs
                                        .character_attrs
                                        .evade_damage
                                        .get() as f64;
                                    (evade_damage != 0.0)
                                        .then(move || {
                                            view! {
                                                <Stat
                                                    label="Evaded Damage over Time Taken"
                                                    value=move || format!("{:.0}%", evade_damage)
                                                />
                                            }
                                        })
                                }}
                                {move || {
                                    let take_from_mana_before_life = game_context
                                        .player_specs
                                        .read()
                                        .character_specs
                                        .character_attrs
                                        .take_from_mana_before_life
                                        .get() as f64;
                                    (take_from_mana_before_life != 0.0)
                                        .then(move || {
                                            view! {
                                                <Stat
                                                    label="Mana Taken Before Life"
                                                    value=move || format!("{:.0}%", take_from_mana_before_life)
                                                />
                                            }
                                        })
                                }}
                                {move || {
                                    let take_from_life_before_mana = game_context
                                        .player_specs
                                        .read()
                                        .character_specs
                                        .character_attrs
                                        .take_from_life_before_mana
                                        .get() as f64;
                                    (take_from_life_before_mana != 0.0)
                                        .then(move || {
                                            view! {
                                                <Stat
                                                    label="Life Taken Before Mana"
                                                    value=move || format!("{:.0}%", take_from_life_before_mana)
                                                />
                                            }
                                        })
                                }}
                                {move || {
                                    DamageType::iter()
                                        .filter_map(|damage_type| {
                                            let evade = game_context
                                                .player_specs
                                                .read()
                                                .character_specs
                                                .character_attrs
                                                .evade
                                                .get(&damage_type)
                                                .copied()
                                                .unwrap_or_default();
                                            (evade.value.get() != 0.0
                                                && damage_type != DamageType::Storm)
                                                .then(|| {
                                                    view! {
                                                        <Stat
                                                            label=format!(
                                                                "{} Evade Chance (max 80%)",
                                                                effects_tooltip::damage_over_time_type_str(
                                                                    Some(damage_type),
                                                                ),
                                                            )
                                                            value=move || { format_chance(&evade) }
                                                        />
                                                    }
                                                })
                                        })
                                        .collect::<Vec<_>>()
                                }}
                                {move || {
                                    DamageType::iter()
                                        .filter_map(|damage_type| {
                                            let value = game_context
                                                .player_specs
                                                .read()
                                                .character_specs
                                                .character_attrs
                                                .damage_taken
                                                .get(&(SkillType::Attack, damage_type))
                                                .map(|x| **x)
                                                .unwrap_or(100.0);
                                            (value != 100.0)
                                                .then(|| {
                                                    view! {
                                                        <Stat
                                                            label=format!(
                                                                "{}Damage Taken",
                                                                effects_tooltip::damage_type_str(Some(damage_type)),
                                                            )
                                                            value=move || { format!("{}%", format_number(value.abs())) }
                                                        />
                                                    }
                                                })
                                        })
                                        .collect::<Vec<_>>()
                                }}
                            </StatCategory>

                            <StatCategory title="Utility">
                                <Stat
                                    label="Threat Gain"
                                    value=move || {
                                        format!(
                                            "{:.0}%",
                                            game_context.player_specs.read().threat_gain.get(),
                                        )
                                    }
                                />
                                <OptionalMultiplierStat
                                    label="Life Restore"
                                    multiplier=move || {
                                        compute_stats_effects_restore_value(
                                            &effects_map.read(),
                                            RestoreType::Life,
                                        )
                                    }
                                />
                                <OptionalMultiplierStat
                                    label="Mana Restore"
                                    multiplier=move || {
                                        compute_stats_effects_restore_value(
                                            &effects_map.read(),
                                            RestoreType::Mana,
                                        )
                                    }
                                />
                                {[
                                    SkillType::Attack,
                                    SkillType::Spell,
                                    SkillType::Curse,
                                    SkillType::Blessing,
                                ]
                                    .into_iter()
                                    .map(|skill_type| {
                                        let label = format_multiplier_stat_name(
                                            &StatType::ManaCost {
                                                skill_filter: StatSkillFilter {
                                                    skill_type: Some(skill_type),
                                                    ..Default::default()
                                                },
                                            },
                                        );
                                        view! {
                                            <OptionalMultiplierStat
                                                label=label
                                                multiplier=move || {
                                                    compute_stats_effects_mana_cost_value(
                                                        &effects_map.read(),
                                                        skill_type,
                                                    )
                                                }
                                            />
                                        }
                                    })
                                    .collect_view()}
                                {[SkillType::Blessing, SkillType::Curse]
                                    .into_iter()
                                    .flat_map(|skill_type| {
                                        let skill_filter = StatSkillFilter {
                                            skill_type: Some(skill_type),
                                            ..Default::default()
                                        };
                                        let power_label = format_multiplier_stat_name(
                                            &StatType::StatusPower {
                                                status_filter: Default::default(),
                                                skill_filter: skill_filter.clone(),
                                                min_max: None,
                                            },
                                        );
                                        let duration_label = format_multiplier_stat_name(
                                            &StatType::StatusDuration {
                                                status_filter: Default::default(),
                                                skill_filter,
                                            },
                                        );
                                        [
                                            view! {
                                                <OptionalMultiplierStat
                                                    label=power_label
                                                    multiplier=move || {
                                                        compute_stats_effects_status_power_value(
                                                            &effects_map.read(),
                                                            skill_type,
                                                        )
                                                    }
                                                />
                                            }
                                                .into_any(),
                                            view! {
                                                <OptionalMultiplierStat
                                                    label=duration_label
                                                    multiplier=move || {
                                                        compute_stats_effects_status_duration_value(
                                                            &effects_map.read(),
                                                            skill_type,
                                                        )
                                                    }
                                                />
                                            }
                                                .into_any(),
                                        ]
                                    })
                                    .collect_view()}
                                {[
                                    SkillType::Attack,
                                    SkillType::Spell,
                                    SkillType::Curse,
                                    SkillType::Blessing,
                                ]
                                    .into_iter()
                                    .map(|skill_type| {
                                        view! {
                                            <OptionalMultiplierStat
                                                label=format!(
                                                    "{}Success Chance",
                                                    skill_type_str(Some(skill_type)),
                                                )
                                                multiplier=move || {
                                                    compute_stats_effects_success_chance_value(
                                                        &effects_map.read(),
                                                        skill_type,
                                                    )
                                                }
                                            />
                                        }
                                    })
                                    .collect_view()}
                            </StatCategory>

                            <StatCategory title="Combat">
                                <MultiplierStat
                                    label="Attack Speed"
                                    multiplier=move || {
                                        compute_stats_effects_speed_value(
                                            &effects_map.read(),
                                            SkillType::Attack,
                                        )
                                    }
                                />
                                <MultiplierStat
                                    label="Spell Speed"
                                    multiplier=move || {
                                        compute_stats_effects_speed_value(
                                            &effects_map.read(),
                                            SkillType::Spell,
                                        )
                                    }
                                />
                                <OptionalMultiplierStat
                                    label="Attack Critical Hit Chance"
                                    multiplier=move || {
                                        compute_stats_effects_crit_chance_value(
                                            &effects_map.read(),
                                            SkillType::Attack,
                                        )
                                    }
                                />
                                <OptionalMultiplierStat
                                    label="Spell Critical Hit Chance"
                                    multiplier=move || {
                                        compute_stats_effects_crit_chance_value(
                                            &effects_map.read(),
                                            SkillType::Spell,
                                        )
                                    }
                                />
                                <OptionalMultiplierStat
                                    label="Attack Critical Hit Damage"
                                    multiplier=move || {
                                        compute_stats_effects_crit_damage_value(
                                            &effects_map.read(),
                                            SkillType::Attack,
                                        )
                                    }
                                />
                                <OptionalMultiplierStat
                                    label="Spell Critical Hit Damage"
                                    multiplier=move || {
                                        compute_stats_effects_crit_damage_value(
                                            &effects_map.read(),
                                            SkillType::Spell,
                                        )
                                    }
                                />
                            </StatCategory>
                            <StatCategory title="Damage">
                                <For
                                    each=move || damage_multipliers.get()
                                    key=|stat| stat.kind.clone()
                                    let:stat
                                >
                                    <MultiplierStat
                                        label=stat.label
                                        multiplier=move || stat.multiplier
                                    />
                                </For>
                            </StatCategory>

                        </div>

                        <TriggersStats />
                    </div>

                </MenuCard>
            </Show>
        </MenuPanel>
    }
}
#[component]
fn StatCategory(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <CardInset pad=false>
            // <h2 class="text-amber-300 text-sm xl:text-base font-bold mb-1 xl:mb-2 tracking-wide">
            // {title}
            // </h2>
            <CardInsetTitle>{title}</CardInsetTitle>
            <div class="flex flex-col gap-1 stat-list">{children()}</div>
        </CardInset>
    }
}

#[component]
fn Stat(
    #[prop(into)] label: String,
    value: impl Fn() -> String + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="flex justify-between px-6 text-sm xl:text-base">
            <span class="text-zinc-400">{label}</span>
            <span class="text-amber-100 font-medium font-number">{move || value()}</span>
        </div>
    }
}

#[component]
fn MultiplierStat(
    #[prop(into)] label: String,
    multiplier: impl Fn() -> f64 + Send + Sync + 'static,
) -> impl IntoView {
    let value = Signal::derive(move || multiplier() * 100.0);

    view! {
        <div class="flex justify-between px-6 text-sm xl:text-base">
            <span class="text-zinc-400">{label}</span>
            <span class="text-amber-100 font-medium font-number">
                <Number value=value />
                "%"
            </span>
        </div>
    }
}

#[component]
fn OptionalMultiplierStat(
    #[prop(into)] label: String,
    multiplier: impl Fn() -> f64 + Send + Sync + 'static,
) -> impl IntoView {
    let multiplier = Signal::derive(multiplier);

    view! {
        <Show when=move || { (multiplier.get() - 1.0).abs() > f64::EPSILON }>
            <MultiplierStat label=label.clone() multiplier=move || multiplier.get() />
        </Show>
    }
}

fn format_hit_damage_kind(kind: HitDamageKind) -> String {
    let damage_type = effects_tooltip::damage_type_str(Some(kind.damage_type));
    let skill_type = skill_type_str(Some(kind.skill_type));
    format!("{damage_type}{skill_type}Hit Damage")
}

#[component]
fn TriggersStats() -> impl IntoView {
    let game_context = expect_context::<GameContext>();
    let triggers = Memo::new(move |_| {
        let mut triggers = game_context
            .player_specs
            .read()
            .character_specs
            .triggers
            .iter()
            .flat_map(|(event_trigger, owned_triggers)| {
                owned_triggers.iter().map(|owned_trigger| {
                    (event_trigger.clone(), owned_trigger.trigger_effect.clone())
                })
            })
            .collect::<Vec<_>>();
        triggers.sort_by_key(|(_, trigger_effect)| trigger_effect.trigger_id.clone());
        triggers
    });

    // after:absolute after:left-0 after:right-0 after:bottom-0 after:h-px
    // after:bg-gradient-to-r after:from-transparent after:via-zinc-600 after:to-transparent
    // last:after:hidden
    // xl:[&:nth-last-child(-n+3)]:after:hidden
    // [&:nth-last-child(-n+2)]:after:hidden
    view! {
        <CardInset pad=false class="w-full min-h-0">
            <CardInsetTitle>"Triggered Effects"</CardInsetTitle>
            <div class="columns-2 xl:columns-3 gap-1">
                <For
                    each=move || triggers.get().into_iter()
                    key=|(trigger, trigger_effect)| {
                        (trigger.clone(), trigger_effect.trigger_id.clone())
                    }
                    let((trigger, trigger_effect))
                >
                    <div class="relative pb-2 list-none break-inside-avoid">
                        {move || {
                            trigger_tooltip::format_trigger(
                                TriggerSpecs {
                                    trigger: trigger.clone(),
                                    description: None,
                                    trigger_effect: trigger_effect.clone(),
                                },
                                true,
                                None,
                                None,
                            )
                        }} <Separator />
                    </div>
                </For>

            </div>
        </CardInset>
    }
}

pub fn format_chance(chance: &BoundedChance) -> String {
    let luck_chance = chance
        .luck_estimate()
        .map(|luck_estimate| format!(" ({:.0}%)", luck_estimate))
        .unwrap_or_default();

    format!("{:.0}%{luck_chance}", chance.value.get())
}
