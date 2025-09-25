use leptos::{html::*, prelude::*};

use shared::data::{
    skill::{DamageType, SkillType},
    stat_effect::{Modifier, StatType},
};

use crate::components::{
    game::GameContext,
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
                                label="Increased Storm Damage"
                                value=move || {
                                    format!(
                                        "+{}%",
                                        format_number(
                                            effect(
                                                StatType::Damage {
                                                    skill_type: None,
                                                    damage_type: Some(DamageType::Storm),
                                                },
                                                Modifier::Multiplier,
                                            ) * 100.0,
                                        ),
                                    )
                                }
                            />
                            <Stat
                                label="Increased Critical Chance"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(StatType::CritChance(None), Modifier::Multiplier)
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
                                        game_context.player_specs.read().character_specs.block,
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
        <div class="bg-neutral-900 rounded-lg shadow-[inset_0_0_24px_rgba(0,0,0,0.6)]
        py-2 xl:py-4 ring-1 ring-zinc-900">
            <h2 class="text-amber-300 text-sm xl:text-base font-bold mb-1 xl:mb-2 tracking-wide">
                {title}
            </h2>
            <div class="flex flex-col gap-1 stat-list">{children()}</div>
        </div>
    }
}

#[component]
fn Stat(label: &'static str, value: impl Fn() -> String + 'static) -> impl IntoView {
    view! {
        <div class="flex justify-between px-6 text-sm xl:text-base">
            <span class="text-gray-400">{label}</span>
            <span class="text-amber-100 font-medium">{value()}</span>
        </div>
    }
}
