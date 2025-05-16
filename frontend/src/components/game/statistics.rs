use leptos::html::*;
use leptos::prelude::*;
use std::time::Duration;

use shared::data::{
    effect::{EffectModifier, EffectTarget},
    skill::DamageType,
};

use crate::components::ui::{buttons::CloseButton, menu_panel::MenuPanel, number::format_number};

use super::game_context::GameContext;

#[component]
pub fn StatisticsPanel(open: RwSignal<bool>) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let stats = move || game_context.game_stats.read();
    let effect = move |stat: EffectTarget, modifier: EffectModifier| {
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
                <div class="bg-zinc-800 rounded-md p-6 shadow-xl ring-1 ring-zinc-950 flex flex-col gap-4">
                    <div class="px-4 relative z-10 flex items-center justify-between">
                        <span class="text-shadow-md shadow-gray-950 text-amber-200 text-xl font-semibold">
                            "Statistics "
                        </span>
                        <CloseButton on:click=move |_| open.set(false) />
                    </div>

                    <div class="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-2">

                        <StatCategory title="Game Statistics">
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
                                    game_context
                                        .player_specs
                                        .read()
                                        .character_specs
                                        .max_life
                                        .to_string()
                                }
                            />
                            <Stat
                                label="Life Regeneration"
                                value=move || {
                                    format!(
                                        "{:.1} per second",
                                        game_context.player_specs.read().character_specs.life_regen,
                                    )
                                }
                            />
                            <Stat
                                label="Maximum Mana"
                                value=move || {
                                    game_context.player_specs.read().max_mana.to_string()
                                }
                            />
                            <Stat
                                label="Mana Regeneration"
                                value=move || {
                                    format!(
                                        "{:.1} per second",
                                        game_context.player_specs.read().mana_regen,
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
                                label="Increased Speed"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(
                                            EffectTarget::GlobalSpeed,
                                            EffectModifier::Multiplier,
                                        ) * 100.0,
                                    )
                                }
                            />
                            <Stat
                                label="Increased Physical Damage"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(
                                            EffectTarget::GlobalDamage(DamageType::Physical),
                                            EffectModifier::Multiplier,
                                        ) * 100.0,
                                    )
                                }
                            />
                            <Stat
                                label="Increased Fire Damage"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(
                                            EffectTarget::GlobalDamage(DamageType::Fire),
                                            EffectModifier::Multiplier,
                                        ) * 100.0,
                                    )
                                }
                            />
                            <Stat
                                label="Increased Poison Damage"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(
                                            EffectTarget::GlobalDamage(DamageType::Poison),
                                            EffectModifier::Multiplier,
                                        ) * 100.0,
                                    )
                                }
                            />
                            <Stat
                                label="Increased Critical Chances"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(
                                            EffectTarget::GlobalCritChances,
                                            EffectModifier::Multiplier,
                                        ) * 100.0,
                                    )
                                }
                            />
                            <Stat
                                label="Increased Critical Damage"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(
                                            EffectTarget::GlobalCritDamage,
                                            EffectModifier::Multiplier,
                                        ) * 100.0,
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
                                label="Fire Armor"
                                value=move || {
                                    format!(
                                        "{:.0}",
                                        game_context.player_specs.read().character_specs.fire_armor,
                                    )
                                }
                            />
                            <Stat
                                label="Poison Armor"
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
                        </StatCategory>

                        <StatCategory title="Attacks">
                            <Stat
                                label="Increased Attack Damage"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(
                                            EffectTarget::GlobalAttackDamage,
                                            EffectModifier::Multiplier,
                                        ) * 100.0,
                                    )
                                }
                            />
                            <Stat
                                label="Increased Attack Speed"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(
                                            EffectTarget::GlobalAttackSpeed,
                                            EffectModifier::Multiplier,
                                        ) * 100.0,
                                    )
                                }
                            />
                        </StatCategory>

                        <StatCategory title="Spells">
                            <Stat
                                label="Increased Spell Power"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(
                                            EffectTarget::GlobalSpellPower,
                                            EffectModifier::Multiplier,
                                        ) * 100.0,
                                    )
                                }
                            />
                            <Stat
                                label="Increased Spell Damage"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(
                                            EffectTarget::GlobalSpellDamage,
                                            EffectModifier::Multiplier,
                                        ) * 100.0,
                                    )
                                }
                            />
                            <Stat
                                label="Increased Casting Speed"
                                value=move || {
                                    format!(
                                        "+{:.0}%",
                                        effect(
                                            EffectTarget::GlobalSpellSpeed,
                                            EffectModifier::Multiplier,
                                        ),
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
        <div class="bg-zinc-800 rounded-lg p-4 shadow-[inset_0_0_24px_rgba(0,0,0,0.6)] ring-1 ring-zinc-900">
            <h2 class="text-amber-300 text-lg font-bold mb-2 tracking-wide">{title}</h2>
            <div class="flex flex-col gap-1">{children()}</div>
        </div>
    }
}

#[component]
fn Stat(label: &'static str, value: impl Fn() -> String + 'static) -> impl IntoView {
    view! {
        <div class="flex justify-between">
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
