use leptos::{html::*, prelude::*};

use shared::data::{
    chance::BoundedChance,
    conditional_modifier::Condition,
    modifier::Modifier,
    skill::{DamageType, RestoreType, SkillType},
    stat_effect::{StatStatusType, StatType},
    trigger::TriggerSpecs,
};
use strum::IntoEnumIterator;

use crate::components::{
    game::GameContext,
    shared::tooltips::{
        effects_tooltip::{self, format_multiplier_stat_name},
        trigger_tooltip,
    },
    ui::{
        Separator,
        card::{CardHeader, CardInset, CardInsetTitle, MenuCard},
        menu_panel::MenuPanel,
        number::{format_duration, format_number},
    },
};

#[component]
pub fn StatisticsPanel(open: RwSignal<bool>) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let stats = move || game_context.game_stats.read();
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

    view! {
        <MenuPanel open=open h_full=false center=false>
            <MenuCard>
                <CardHeader title="Statistics" on_close=move || open.set(false) />

                <div class="grid grid-cols-2 xl:grid-cols-3 gap-2 xl:gap-4 overflow-y-auto">

                    <StatCategory title="Game">
                        <Stat
                            label="Elapsed Time"
                            value=move || format_duration(stats().elapsed_time, true)
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
                                format_number(game_context.game_local_stats.average_damage_tick())
                            }
                        />
                        <Stat
                            label="Gold Collected"
                            value=move || {
                                format_number(game_context.player_resources.read().gold_total)
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
                                    game_context.player_specs.read().character_specs.max_life.get(),
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
                                    .life_regen * 0.1;
                                if value == 0.0 { "-".into() } else { format!("{:.1}%", value) }
                            }
                        />
                        <Stat
                            label="Maximum Mana"
                            value=move || {
                                format_number(
                                    game_context.player_specs.read().character_specs.max_mana.get(),
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
                                    .mana_regen * 0.1;
                                if value == 0.0 { "-".into() } else { format!("{:.1}%", value) }
                            }
                        />
                        <Stat
                            label="Gold Find"
                            value=move || {
                                format!(
                                    "{}%",
                                    format_number(game_context.player_specs.read().gold_find.get()),
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
                            label="Physical Armor"
                            value=move || {
                                format!(
                                    "{:.0}",
                                    *game_context
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
                                    *game_context
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
                                    *game_context
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
                                    *game_context
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
                            label="Attack Block Chance (max 80%)"
                            value=move || {
                                format_chance(
                                    &game_context
                                        .player_specs
                                        .read()
                                        .character_specs
                                        .block
                                        .get(&SkillType::Attack)
                                        .copied()
                                        .unwrap_or_default(),
                                )
                            }
                        />
                        {move || {
                            let block_spell = game_context
                                .player_specs
                                .read()
                                .character_specs
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
                                        .evade
                                        .get(&damage_type)
                                        .copied()
                                        .unwrap_or_default();
                                    (evade.value.get() != 0.0 && damage_type != DamageType::Storm)
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
                                    let value = -game_context
                                        .player_specs
                                        .read()
                                        .character_specs
                                        .effects
                                        .0
                                        .get(
                                            &(
                                                StatType::DamageResistance {
                                                    skill_type: None,
                                                    damage_type: Some(damage_type),
                                                },
                                                Modifier::Flat,
                                                false,
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
                                format!(
                                    "{:.0}%",
                                    game_context.player_specs.read().threat_gain.get(),
                                )
                            }
                        />
                        {make_stat(
                            StatType::Restore {
                                restore_type: None,
                                skill_type: None,
                            },
                            Modifier::Increased,
                        )}
                        {make_opt_stat(
                            StatType::Restore {
                                restore_type: Some(RestoreType::Life),
                                skill_type: None,
                            },
                            Modifier::Increased,
                            0.0,
                        )}
                        {make_opt_stat(
                            StatType::Restore {
                                restore_type: Some(RestoreType::Mana),
                                skill_type: None,
                            },
                            Modifier::Increased,
                            0.0,
                        )}
                        {make_stat(
                            StatType::StatusDuration {
                                status_type: None,
                                skill_type: None,
                            },
                            Modifier::Increased,
                        )}
                        {make_stat(
                            StatType::StatusPower {
                                status_type: None,
                                skill_type: None,
                                min_max: None,
                            },
                            Modifier::Increased,
                        )}
                        // TODO: More for stun?
                        {make_opt_stat(
                            StatType::StatusPower {
                                status_type: None,
                                skill_type: Some(SkillType::Blessing),
                                min_max: None,
                            },
                            Modifier::Increased,
                            0.0,
                        )}
                        {make_opt_stat(
                            StatType::StatusDuration {
                                status_type: None,
                                skill_type: Some(SkillType::Blessing),
                            },
                            Modifier::Increased,
                            0.0,
                        )}
                        {make_opt_stat(
                            StatType::StatusPower {
                                status_type: None,
                                skill_type: Some(SkillType::Curse),
                                min_max: None,
                            },
                            Modifier::Increased,
                            0.0,
                        )}
                        {make_opt_stat(
                            StatType::StatusDuration {
                                status_type: None,
                                skill_type: Some(SkillType::Curse),
                            },
                            Modifier::Increased,
                            0.0,
                        )}
                        {make_opt_stat(
                            StatType::SuccessChance {
                                skill_type: None,
                                effect_type: None,
                            },
                            Modifier::Increased,
                            0.0,
                        )}
                    </StatCategory>

                    <StatCategory title="Combat">
                        {make_stat(StatType::Speed(None), Modifier::Increased)}
                        {make_stat(StatType::Speed(Some(SkillType::Attack)), Modifier::Increased)}
                        {make_stat(StatType::Speed(Some(SkillType::Spell)), Modifier::Increased)}
                        {make_stat(StatType::CritChance(None), Modifier::Increased)}
                        {make_opt_stat(
                            StatType::CritChance(Some(SkillType::Spell)),
                            Modifier::Increased,
                            0.0,
                        )}
                        {make_opt_stat(
                            StatType::StatConditionalModifier {
                                stat: Box::new(StatType::Damage {
                                    skill_type: None,
                                    damage_type: None,
                                    min_max: None,
                                }),
                                conditions: vec![Condition::ThreatLevel],
                                conditions_duration: 0,
                            },
                            Modifier::More,
                            0.0,
                        )}
                    </StatCategory>
                    <StatCategory title="Damage">
                        {make_opt_stat(
                            StatType::Damage {
                                skill_type: None,
                                damage_type: None,
                                min_max: None,
                            },
                            Modifier::More,
                            0.0,
                        )}
                        {make_stat(
                            StatType::Damage {
                                skill_type: Some(SkillType::Attack),
                                damage_type: None,
                                min_max: None,
                            },
                            Modifier::More,
                        )}
                        {make_stat(
                            StatType::Damage {
                                skill_type: Some(SkillType::Spell),
                                damage_type: None,
                                min_max: None,
                            },
                            Modifier::More,
                        )}
                        {make_opt_stat(
                            StatType::Damage {
                                skill_type: None,
                                damage_type: Some(DamageType::Physical),
                                min_max: None,
                            },
                            Modifier::More,
                            0.0,
                        )}
                        {make_opt_stat(
                            StatType::Damage {
                                skill_type: None,
                                damage_type: Some(DamageType::Fire),
                                min_max: None,
                            },
                            Modifier::More,
                            0.0,
                        )}
                        {make_opt_stat(
                            StatType::Damage {
                                skill_type: None,
                                damage_type: Some(DamageType::Poison),
                                min_max: None,
                            },
                            Modifier::More,
                            0.0,
                        )}
                        {make_opt_stat(
                            StatType::Damage {
                                skill_type: None,
                                damage_type: Some(DamageType::Storm),
                                min_max: None,
                            },
                            Modifier::More,
                            0.0,
                        )} // TODO: Elemental dot?
                        {make_opt_stat(
                            StatType::StatusPower {
                                status_type: Some(StatStatusType::DamageOverTime {
                                    damage_type: None,
                                }),
                                skill_type: None,
                                min_max: None,
                            },
                            Modifier::More,
                            0.0,
                        )} {make_opt_stat(StatType::CritDamage(None), Modifier::More, 0.0)}
                    </StatCategory>

                    <TriggersStats class:col-span-2 class:xl:col-span-3 />
                </div>

            </MenuCard>
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

fn make_opt_stat(stat_type: StatType, modifier: Modifier, default: f64) -> impl IntoView + use<> {
    let game_context = expect_context::<GameContext>();

    view! {
        {move || {
            let value = game_context
                .player_specs
                .read()
                .character_specs
                .effects
                .0
                .get(&(stat_type.clone(), modifier, false))
                .copied()
                .unwrap_or_default();
            (default != value).then(|| make_stat(stat_type.clone(), modifier))
        }}
    }
}

fn make_stat(stat_type: StatType, modifier: Modifier) -> impl IntoView + use<> {
    let game_context = expect_context::<GameContext>();

    view! {
        <Stat
            label=format!(
                "{} {}",
                effects_tooltip::modifier_str(modifier),
                format_multiplier_stat_name(&stat_type),
            )
            value=move || format_effect_value(
                game_context
                    .player_specs
                    .read()
                    .character_specs
                    .effects
                    .0
                    .get(&(stat_type.clone(), modifier, false))
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

#[component]
fn TriggersStats() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    // after:absolute after:left-0 after:right-0 after:bottom-0 after:h-px
    // after:bg-gradient-to-r after:from-transparent after:via-zinc-600 after:to-transparent
    // last:after:hidden
    // xl:[&:nth-last-child(-n+3)]:after:hidden
    // [&:nth-last-child(-n+2)]:after:hidden
    view! {
        <CardInset pad=false class="w-full">
            <CardInsetTitle>"Triggered Effects"</CardInsetTitle>
            <div class="columns-2 xl:columns-3 gap-1">
                {move || {
                    let mut triggers = game_context
                        .player_specs
                        .read()
                        .character_specs
                        .triggers
                        .clone();
                    triggers.sort_by_key(|trigger| trigger.trigger_id.clone());

                    view! {
                        <For
                            each=move || triggers.clone().into_iter()
                            key=|triggered_effect| triggered_effect.trigger_id.clone()
                            let(triggered_effect)
                        >
                            <div class="relative pb-2 list-none break-inside-avoid">
                                {trigger_tooltip::format_trigger(TriggerSpecs {
                                    name: None,
                                    icon: None,
                                    description: None,
                                    triggered_effect: triggered_effect.clone(),
                                    is_debuff: false,
                                })} <Separator />
                            </div>
                        </For>
                    }
                }}

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
