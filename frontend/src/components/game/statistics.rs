use leptos::html::*;
use leptos::prelude::*;
use shared::data::skill::SkillType;
use std::time::Duration;

use shared::data::{
    skill::DamageType,
    stat_effect::{Modifier, StatType},
};

use crate::components::ui::{buttons::CloseButton, menu_panel::MenuPanel, number::format_number};

use super::game_context::GameContext;

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
            <div class="w-full p-4">
                <div class="bg-zinc-800 rounded-md p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col gap-2">
                    <div class="px-4 relative z-10 flex items-center justify-between">
                        <span class="text-shadow-md shadow-gray-950 text-amber-200 text-xl font-semibold">
                            "Statistics "
                        </span>
                        <CloseButton on:click=move |_| open.set(false) />
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-4">

                        <StatCategory title="Game">
                            <Stat
                                label="Elapsed Time"
                                value=move || format_duration(stats().elapsed_time)
                            />
                            <Stat
                                label="Highest Area Level"
                                value=move || stats().highest_area_level.to_string()
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
                                        "{:.2}%",
                                        game_context.player_specs.read().character_specs.life_regen,
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
                                        "{:.2}%",
                                        game_context.player_specs.read().character_specs.mana_regen,
                                    )
                                }
                            />
                            <Stat
                                label="Gold Find"
                                value=move || {
                                    format!(
                                        "{}%",
                                        format_number(
                                            game_context.player_specs.read().gold_find * 100.0,
                                        ),
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

                        <StatCategory title="Damage">
                            <Stat
                                label="Increased Action Speed"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(StatType::Speed(None), Modifier::Multiplier) * 100.0,
                                    )
                                }
                            />
                            <Stat
                                label="Increased Physical Damage"
                                value=move || {
                                    format!(
                                        "+{}%",
                                        format_number(
                                            effect(
                                                StatType::Damage {
                                                    skill_type: None,
                                                    damage_type: Some(DamageType::Physical),
                                                },
                                                Modifier::Multiplier,
                                            ) * 100.0,
                                        ),
                                    )
                                }
                            />
                            <Stat
                                label="Increased Fire Damage"
                                value=move || {
                                    format!(
                                        "+{}%",
                                        format_number(
                                            effect(
                                                StatType::Damage {
                                                    skill_type: None,
                                                    damage_type: Some(DamageType::Fire),
                                                },
                                                Modifier::Multiplier,
                                            ) * 100.0,
                                        ),
                                    )
                                }
                            />
                            <Stat
                                label="Increased Poison Damage"
                                value=move || {
                                    format!(
                                        "+{}%",
                                        format_number(
                                            effect(
                                                StatType::Damage {
                                                    skill_type: None,
                                                    damage_type: Some(DamageType::Poison),
                                                },
                                                Modifier::Multiplier,
                                            ) * 100.0,
                                        ),
                                    )
                                }
                            />
                            <Stat
                                label="Increased Critical Chances"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(StatType::CritChances(None), Modifier::Multiplier)
                                            * 100.0,
                                    )
                                }
                            />
                            <Stat
                                label="Increased Critical Damage"
                                value=move || {
                                    format!(
                                        "+{}%",
                                        format_number(
                                            effect(StatType::CritDamage(None), Modifier::Multiplier)
                                                * 100.0,
                                        ),
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
                                        game_context.player_specs.read().character_specs.armor,
                                    )
                                }
                            />
                            <Stat
                                label="Fire Resistance"
                                value=move || {
                                    format!(
                                        "{:.0}",
                                        game_context.player_specs.read().character_specs.fire_armor,
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
                                            .poison_armor,
                                    )
                                }
                            />
                            <Stat
                                label="Block Chances"
                                value=move || {
                                    format!(
                                        "{:.0}%",
                                        game_context.player_specs.read().character_specs.block
                                            * 100.0,
                                    )
                                }
                            />
                        </StatCategory>

                        <StatCategory title="Attacks">
                            <Stat
                                label="Increased Attack Speed"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(
                                            StatType::Speed(Some(SkillType::Attack)),
                                            Modifier::Multiplier,
                                        ) * 100.0,
                                    )
                                }
                            />
                            <Stat
                                label="Increased Attack Damage"
                                value=move || {
                                    format!(
                                        "+{}%",
                                        format_number(
                                            effect(
                                                StatType::Damage {
                                                    skill_type: Some(SkillType::Attack),
                                                    damage_type: None,
                                                },
                                                Modifier::Multiplier,
                                            ) * 100.0,
                                        ),
                                    )
                                }
                            />
                        </StatCategory>

                        <StatCategory title="Spells">
                            <Stat
                                label="Increased Casting Speed"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(
                                            StatType::Speed(Some(SkillType::Spell)),
                                            Modifier::Multiplier,
                                        ) * 100.0,
                                    )
                                }
                            />
                            <Stat
                                label="Increased Spell Damage"
                                value=move || {
                                    format!(
                                        "+{}%",
                                        format_number(
                                            effect(
                                                StatType::Damage {
                                                    skill_type: Some(SkillType::Spell),
                                                    damage_type: None,
                                                },
                                                Modifier::Multiplier,
                                            ) * 100.0,
                                        ),
                                    )
                                }
                            />
                            <Stat
                                label="Increased Spell Power"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(StatType::SpellPower, Modifier::Multiplier) * 100.0,
                                    )
                                }
                            />
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
        <div class="bg-neutral-900 rounded-lg shadow-[inset_0_0_24px_rgba(0,0,0,0.6)]  py-4 ring-1 ring-zinc-900">
            <h2 class="text-amber-300 text-md font-bold mb-2 tracking-wide">{title}</h2>
            <div class="flex flex-col gap-1 stat-list">{children()}</div>
        </div>
    }
}

#[component]
fn Stat(label: &'static str, value: impl Fn() -> String + 'static) -> impl IntoView {
    view! {
        <div class="flex justify-between px-6">
            <span class="text-gray-300">{label}</span>
            <span class="text-amber-100 font-medium">{value()}</span>
        </div>
    }
}

fn format_duration(dur: Duration) -> String {
    let secs = dur.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
}
