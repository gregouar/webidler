use leptos::{html::*, prelude::*};

use shared::{
    computations, constants,
    data::{
        skill::{DamageType, RestoreType, SkillType},
        stat_effect::{
            Modifier, StatConverterSource, StatConverterSpecs, StatStatusType, StatType,
        },
        trigger::HitTrigger,
    },
};
use strum::IntoEnumIterator;

use crate::components::{
    game::GameContext,
    shared::tooltips::effects_tooltip::{self, format_multiplier_stat_name},
    ui::{
        buttons::CloseButton,
        menu_panel::{MenuPanel, PanelTitle},
        number::{format_duration, format_number},
    },
};

#[component]
pub fn StatisticsPanel(open: RwSignal<bool>) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let stats = move || game_context.game_stats.read();
    let effect = move |stat: StatType, modifier: Modifier| {
        game_context
            .player_specs
            .read()
            .effects
            .0
            .get(&(stat, modifier))
            .copied()
            .unwrap_or_default()
    };

    view! {
        <MenuPanel open=open>
            <div class="w-full  h-full">
                <div class="bg-zinc-800 rounded-md p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col gap-2 max-h-full">
                    <div class="px-4 relative z-10 flex items-center justify-between">
                        <PanelTitle>"Statistics "</PanelTitle>
                        <CloseButton on:click=move |_| open.set(false) />
                    </div>

                    <div class="grid grid-cols-2 xl:grid-cols-3 gap-2 xl:gap-4 overflow-y-auto">

                        <StatCategory title="Game">
                            <Stat
                                label="Elapsed Time"
                                value=move || format_duration(stats().elapsed_time)
                            />
                            <Stat
                                label="Areas Completed"
                                value=move || stats().areas_completed.to_string()
                            />
                            <Stat
                                label="Monsters Killed"
                                value=move || stats().monsters_killed.to_string()
                            />
                            <Stat
                                label="Player Deaths"
                                value=move || stats().player_deaths.to_string()
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
                                    game_context.area_state.read().max_area_level_ever.to_string()
                                }
                            />
                            <Stat
                                label="Average Damage per Second"
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
                            <Stat
                                label="Gold Collected"
                                value=move || {
                                    format_number(game_context.player_resources.read().gold_total)
                                }
                            />
                            <Stat
                                label="Gold Donations Collected"
                                value=move || {
                                    format_number(
                                        game_context.player_resources.read().gold_total
                                            * computations::exponential(
                                                game_context.area_specs.read().item_level_modifier,
                                                constants::MONSTER_INCREASE_FACTOR,
                                            ),
                                    )
                                }
                            />
                        </StatCategory>

                        <StatCategory title="Character">
                            <Stat
                                label="Name"
                                value=move || {
                                    game_context.player_specs.read().character_specs.name.clone()
                                }
                            />
                            <Stat
                                label="Level"
                                value=move || game_context.player_specs.read().level.to_string()
                            />
                            <Stat
                                label="Maximum Life"
                                value=move || {
                                    format_number(
                                        game_context.player_specs.read().character_specs.max_life,
                                    )
                                }
                            />
                            <Stat
                                label="Life Regeneration per second"
                                value=move || {
                                    format!(
                                        "{:.1}%",
                                        game_context.player_specs.read().character_specs.life_regen
                                            * 0.1,
                                    )
                                }
                            />
                            <Stat
                                label="Maximum Mana"
                                value=move || {
                                    format_number(
                                        game_context.player_specs.read().character_specs.max_mana,
                                    )
                                }
                            />
                            <Stat
                                label="Mana Regeneration per second"
                                value=move || {
                                    format!(
                                        "{:.1}%",
                                        game_context.player_specs.read().character_specs.mana_regen
                                            * 0.1,
                                    )
                                }
                            />
                            <Stat
                                label="Gold Find"
                                value=move || {
                                    format!(
                                        "{}%",
                                        format_number(game_context.player_specs.read().gold_find),
                                    )
                                }
                            />
                            <Stat
                                label="Movement Cooldown"
                                value=move || {
                                    format!(
                                        "{:.2}s",
                                        game_context.player_specs.read().movement_cooldown,
                                    )
                                }
                            />
                        </StatCategory>

                        <StatCategory title="Defense">
                            <Stat
                                label="Physical Armor"
                                value=move || {
                                    format!(
                                        "{:.0}",
                                        game_context
                                            .player_specs
                                            .read()
                                            .character_specs
                                            .armor
                                            .get(&DamageType::Physical)
                                            .cloned()
                                            .unwrap_or_default(),
                                    )
                                }
                            />
                            <Stat
                                label="Fire Resistance"
                                value=move || {
                                    format!(
                                        "{:.0}",
                                        game_context
                                            .player_specs
                                            .read()
                                            .character_specs
                                            .armor
                                            .get(&DamageType::Fire)
                                            .cloned()
                                            .unwrap_or_default(),
                                    )
                                }
                            />
                            <Stat
                                label="Poison Resistance"
                                value=move || {
                                    format!(
                                        "{:.0}",
                                        game_context
                                            .player_specs
                                            .read()
                                            .character_specs
                                            .armor
                                            .get(&DamageType::Poison)
                                            .cloned()
                                            .unwrap_or_default(),
                                    )
                                }
                            />
                            <Stat
                                label="Storm Resistance"
                                value=move || {
                                    format!(
                                        "{:.0}",
                                        game_context
                                            .player_specs
                                            .read()
                                            .character_specs
                                            .armor
                                            .get(&DamageType::Storm)
                                            .cloned()
                                            .unwrap_or_default(),
                                    )
                                }
                            />
                            <Stat
                                label="Block Chance"
                                value=move || {
                                    format!(
                                        "{:.0}%",
                                        game_context.player_specs.read().character_specs.block.value,
                                    )
                                }
                            />
                            {move || {
                                let block_spell = game_context
                                    .player_specs
                                    .read()
                                    .character_specs
                                    .block_spell
                                    .value as f64;
                                (block_spell != 0.0)
                                    .then(move || {
                                        view! {
                                            <Stat
                                                label="Block Chance Applied to Spells"
                                                value=move || format!("{:.0}%", block_spell)
                                            />
                                        }
                                    })
                            }}
                            {move || {
                                let block_damage = game_context
                                    .player_specs
                                    .read()
                                    .character_specs
                                    .block_damage as f64;
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
                                let take_from_mana_before_life = game_context
                                    .player_specs
                                    .read()
                                    .character_specs
                                    .take_from_mana_before_life as f64;
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
                            // {move || {
                            // game_context
                            // .player_specs
                            // .with(|player_specs| {
                            // view! {
                            // {itertools::iproduct!(
                            // DamageType::iter(),SkillType::iter().collect::<Vec::<_>>()
                            // )
                            // .filter_map(|(damage_type, skill_type)| {
                            // player_specs
                            // .character_specs
                            // .damage_resistance
                            // .get(&(skill_type, damage_type))
                            // .cloned()
                            // .map(|value| {
                            // view! {
                            // <Stat
                            // label=format_multiplier_stat_name(
                            // &StatType::DamageResistance {
                            // skill_type: Some(skill_type),
                            // damage_type: Some(damage_type),
                            // },
                            // )
                            // value=move || format!("{:.0}%", value)
                            // />
                            // }
                            // })
                            // })
                            // .collect::<Vec<_>>()}
                            // }
                            // })
                            // }}
                            {move || {
                                DamageType::iter()
                                    .filter_map(|damage_type| {
                                        let value = -game_context
                                            .player_specs
                                            .read()
                                            .effects
                                            .0
                                            .get(
                                                &(
                                                    StatType::DamageResistance {
                                                        skill_type: None,
                                                        damage_type: Some(damage_type),
                                                    },
                                                    Modifier::Flat,
                                                ),
                                            )
                                            .copied()
                                            .unwrap_or_default();
                                        (value != 0.0)
                                            .then(|| {
                                                view! {
                                                    <Stat
                                                        label=format!(
                                                            "{}Damage Taken",
                                                            effects_tooltip::damage_type_str(Some(damage_type)),
                                                        )
                                                        value=move || format_effect_value(value)
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
                                    format!("{:.0}%", game_context.player_specs.read().threat_gain)
                                }
                            />
                            {move || {
                                let life_on_hit = effect(
                                    StatType::LifeOnHit(HitTrigger {
                                        skill_type: Some(SkillType::Attack),
                                        range: None,
                                        is_crit: None,
                                        is_blocked: None,
                                        is_hurt: Some(true),
                                        is_triggered: None,
                                    }),
                                    Modifier::Flat,
                                );
                                (life_on_hit > 0.0)
                                    .then(move || {
                                        view! {
                                            <Stat
                                                label="Life on Hit"
                                                value=move || { format!("{:.0}", life_on_hit) }
                                            />
                                        }
                                    })
                            }}
                            {move || {
                                let mana_on_hit = effect(
                                    StatType::ManaOnHit(HitTrigger {
                                        skill_type: Some(SkillType::Attack),
                                        range: None,
                                        is_crit: None,
                                        is_blocked: None,
                                        is_hurt: Some(true),
                                        is_triggered: None,
                                    }),
                                    Modifier::Flat,
                                );
                                (mana_on_hit > 0.0)
                                    .then(move || {
                                        view! {
                                            <Stat
                                                label="Mana on Hit"
                                                value=move || { format!("{:.0}", mana_on_hit) }
                                            />
                                        }
                                    })
                            }}
                            {make_stat(StatType::Restore(None))}
                            {make_opt_stat(StatType::Restore(Some(RestoreType::Life)), 0.0)}
                            {make_opt_stat(StatType::Restore(Some(RestoreType::Mana)), 0.0)}
                            {make_stat(StatType::StatusDuration(None))}
                            {make_stat(StatType::StatusPower(None))}
                            {make_opt_stat(
                                StatType::StatusPower(
                                    Some(StatStatusType::StatModifier {
                                        debuff: Some(false),
                                    }),
                                ),
                                0.0,
                            )}
                            {make_opt_stat(
                                StatType::StatusPower(
                                    Some(StatStatusType::StatModifier {
                                        debuff: Some(true),
                                    }),
                                ),
                                0.0,
                            )}
                            {make_opt_stat(
                                StatType::SuccessChance {
                                    skill_type: None,
                                    effect_type: None,
                                },
                                0.0,
                            )}
                        </StatCategory>

                        <StatCategory title="Combat">
                            {make_stat(StatType::Speed(None))}
                            {make_stat(StatType::Speed(Some(SkillType::Attack)))}
                            {make_stat(StatType::Speed(Some(SkillType::Spell)))}
                            {make_stat(StatType::CritChance(None))}
                            {make_opt_stat(StatType::CritChance(Some(SkillType::Spell)), 0.0)}
                            {make_opt_stat(
                                StatType::StatConverter(StatConverterSpecs {
                                    source: StatConverterSource::ThreatLevel,
                                    target_stat: Box::new(StatType::Damage {
                                        skill_type: None,
                                        damage_type: None,
                                    }),
                                    target_modifier: Modifier::Multiplier,
                                    is_extra: false,
                                    skill_type: None,
                                }),
                                0.0,
                            )}
                        </StatCategory>
                        <StatCategory title="Damage">
                            {make_opt_stat(
                                StatType::Damage {
                                    skill_type: None,
                                    damage_type: None,
                                },
                                0.0,
                            )}
                            {make_stat(StatType::Damage {
                                skill_type: Some(SkillType::Attack),
                                damage_type: None,
                            })}
                            {make_stat(StatType::Damage {
                                skill_type: Some(SkillType::Spell),
                                damage_type: None,
                            })}
                            {make_opt_stat(
                                StatType::Damage {
                                    skill_type: None,
                                    damage_type: Some(DamageType::Physical),
                                },
                                0.0,
                            )}
                            {make_opt_stat(
                                StatType::Damage {
                                    skill_type: None,
                                    damage_type: Some(DamageType::Fire),
                                },
                                0.0,
                            )}
                            {make_opt_stat(
                                StatType::Damage {
                                    skill_type: None,
                                    damage_type: Some(DamageType::Poison),
                                },
                                0.0,
                            )}
                            {make_opt_stat(
                                StatType::Damage {
                                    skill_type: None,
                                    damage_type: Some(DamageType::Storm),
                                },
                                0.0,
                            )}
                            {make_opt_stat(
                                StatType::StatusPower(
                                    Some(StatStatusType::DamageOverTime {
                                        damage_type: None,
                                    }),
                                ),
                                0.0,
                            )} {make_opt_stat(StatType::CritDamage(None), 0.0)}
                        </StatCategory>
                    </div>

                </div>
            </div>
        </MenuPanel>
    }
}
#[component]
fn StatCategory(title: &'static str, children: Children) -> impl IntoView {
    view! {
        <style>
            "
            .stat-list > div:nth-child(odd) {
            background-color: rgba(63, 63, 70, 0.2);
            }
            "
        </style>
        <div class="bg-neutral-900 rounded-lg shadow-[inset_0_0_24px_rgba(0,0,0,0.6)]
        py-2 xl:py-4 ring-1 ring-zinc-900">
            <h2 class="text-amber-300 text-sm xl:text-base font-bold mb-1 xl:mb-2 tracking-wide">
                {title}
            </h2>
            <div class="flex flex-col gap-1 stat-list">{children()}</div>
        </div>
    }
}

fn make_opt_stat(stat_type: StatType, default: f64) -> impl IntoView + use<> {
    let game_context = expect_context::<GameContext>();

    view! {
        {move || {
            let value = game_context
                .player_specs
                .read()
                .effects
                .0
                .get(&(stat_type.clone(), Modifier::Multiplier))
                .copied()
                .unwrap_or_default();
            (default != value).then(|| make_stat(stat_type.clone()))
        }}
    }
}

fn make_stat(stat_type: StatType) -> impl IntoView + use<> {
    let game_context = expect_context::<GameContext>();

    view! {
        <Stat
            label=format!(
                "{} {}",
                if stat_type.is_multiplicative() { "More" } else { "Increased" },
                format_multiplier_stat_name(&stat_type),
            )
            value=move || format_effect_value(
                game_context
                    .player_specs
                    .read()
                    .effects
                    .0
                    .get(&(stat_type.clone(), Modifier::Multiplier))
                    .copied()
                    .unwrap_or_default(),
            )
        />
    }
}

#[component]
fn Stat(
    #[prop(into)] label: String,
    value: impl Fn() -> String + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class="flex justify-between px-6 text-sm xl:text-base">
            <span class="text-gray-400">{label}</span>
            <span class="text-amber-100 font-medium font-number">{move || value()}</span>
        </div>
    }
}

pub fn format_effect_value(value: f64) -> String {
    if value >= 0.0 {
        format!("+{}%", format_number(value))
    } else {
        let div = (1.0 - value * 0.01).max(0.0);
        format!(
            "-{}%",
            format_number(-(if div != 0.0 { value / div } else { 0.0 }))
        )
    }
}
