use std::sync::Arc;

use leptos::{html::*, prelude::*};

use rand::Rng;

use shared::data::{character::CharacterSize, monster::MonsterSpecs, skill::SkillSpecs};

use crate::assets::img_asset;
use crate::components::{
    game::tooltips::SkillTooltip,
    ui::{
        number::format_number,
        progress_bars::{CircularProgressBar, HorizontalProgressBar},
        tooltip::{
            DynamicTooltipContext, DynamicTooltipPosition, StaticTooltip, StaticTooltipPosition,
        },
    },
};

use super::character::CharacterPortrait;
use super::GameContext;

#[component]
pub fn MonstersGrid() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let all_monsters_dead = Memo::new(move |_| {
        game_context
            .monster_states
            .read()
            .iter()
            .all(|x| !x.character_state.is_alive)
    });

    let flee = Memo::new(move |_| {
        !game_context.player_state.read().character_state.is_alive
            || game_context.world_state.read().going_back > 0
    });

    // TODO: double buffering to allow in and out at the same time
    view! {
        <div class="
        grid grid-rows-2 grid-cols-3 p-2 gap-2 
        items-center
        w-full aspect-[12/7]
        bg-stone-800
        overflow-hidden shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]
        ">
            <style>
                "
                @keyframes monster-fade-in {
                 0% { transform: translateX(100%); opacity: 0; }
                 65% { transform: translateX(0%); opacity: 1; }
                 80% { transform: translateX(5%); }
                 100% { transform: translateX(0%); }
                }
                
                @keyframes monster-fade-out {
                 from { opacity: 1; transform: translateY(0%); }
                 to { opacity: 0; transform: translateY(100%); }
                }
                
                @keyframes monster-flee {
                 0% { transform: translateX(0%); opacity: 1; }
                 100% { transform: translateX(100%); opacity: 0; }
                }
                "
            </style>
            <For
                each=move || game_context.monster_specs.get().into_iter().enumerate()
                // We need a unique key to replace old elements
                key=move |(index, _)| (game_context.monster_wave.get(), *index)
                children=move |(index, specs)| {
                    let animation_delay = format!(
                        "animation-delay: {}s;",
                        rand::rng().random_range(0.0..=0.2f32),
                    );
                    let (x_size, y_size) = specs.character_specs.size.get_xy_size();
                    let (x_pos, y_pos) = (
                        specs.character_specs.position_x,
                        specs.character_specs.position_y,
                    );

                    view! {
                        <div
                            class=format!(
                                "col-span-{} row-span-{} col-start-{} row-start-{} items-center h-full",
                                x_size,
                                y_size,
                                x_pos,
                                y_pos,
                            )
                            style=move || {
                                if all_monsters_dead.get() {
                                    "animation-delay: 0.2s; animation: monster-fade-out 1s ease-in; animation-fill-mode: both; pointer-events: none;"
                                        .to_string()
                                } else if flee.get() {
                                    format!(
                                        "animation: monster-flee 1s ease-out; animation-fill-mode: both; {} pointer-events: none;",
                                        animation_delay,
                                    )
                                } else {
                                    format!(
                                        "animation: monster-fade-in 1s ease-out; animation-fill-mode: both; {}",
                                        animation_delay,
                                    )
                                }
                            }
                        >
                            <MonsterCard specs=specs index=index />
                        </div>
                    }
                }
            />
        </div>
    }
}

#[component]
fn MonsterCard(specs: MonsterSpecs, index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let monster_name = specs.character_specs.name.clone();
    let is_big = match specs.character_specs.size {
        CharacterSize::Small | CharacterSize::Large => false,
        CharacterSize::Huge | CharacterSize::Gargantuan => true,
    };

    let health = Memo::new(move |_| {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|s| s.character_state.life)
            .unwrap_or_default()
    });

    let health_tooltip = move || {
        view! {
            "Health: "
            {format_number(health.get())}
            "/"
            {format_number(specs.character_specs.max_life)}
        }
    };

    let health_percent = Memo::new(move |_| {
        let max_health = specs.character_specs.max_life;
        if max_health > 0.0 {
            (health.get() / specs.character_specs.max_life * 100.0) as f32
        } else {
            0.0
        }
    });

    let is_dead = Memo::new(move |_| {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|x| !x.character_state.is_alive)
            .unwrap_or_default()
    });

    let just_hurt = Memo::new(move |_| {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|x| x.character_state.just_hurt)
            .unwrap_or_default()
    });

    let just_hurt_crit = Memo::new(move |_| {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|x| x.character_state.just_hurt_crit)
            .unwrap_or_default()
    });

    let just_blocked = Memo::new(move |_| {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|x| x.character_state.just_blocked)
            .unwrap_or_default()
    });

    let statuses = Signal::derive(move || {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|x| x.character_state.statuses.clone())
            .unwrap_or_default()
    });

    let gold_reward = RwSignal::new(0.0);
    Effect::new(move |_| {
        if is_dead.get() {
            gold_reward.set(
                game_context
                    .monster_states
                    .read()
                    .get(index)
                    .map(|x| x.gold_reward)
                    .unwrap_or_default(),
            );
        }
    });

    view! {
        <style>
            "
            .gold-text {
                font-weight: bold;
                background: linear-gradient(90deg, #ffeb3b, #fcd34d, #fde68a, #fff);
                background-size: 400%;
                -webkit-background-clip: text;
                -webkit-text-fill-color: transparent;
                animation: goldShimmer 2s infinite linear;
                text-shadow: 0 0 8px rgba(255, 223, 0, 0.9);
            }
            
            .gold-float {
            animation: goldFloat 2.5s ease-out forwards;
            position: absolute;
            }
            
            @keyframes goldShimmer {
                0% { background-position: 0% }
                100% { background-position: 400% }
            }
            
            @keyframes goldFloat {
                0% {
                    opacity: 0;
                    transform: translateY(0) scale(0.9);
                }
                20% {
                    opacity: 1;
                    transform: translateY(-12px) scale(1.1);
                }
                40% {
                    opacity: 1;
                    transform: translateY(-24px) scale(1.1);
                }
                100% {
                    opacity: 0;
                    transform: translateY(-64px) scale(1);
                }
            }"
        </style>
        <div class="grid grid-cols-4 h-full bg-zinc-800 shadow-md rounded-md gap-2 p-2 ring-1 ring-zinc-950">
            <div class="relative flex flex-col gap-2 col-span-3 h-full">
                <StaticTooltip tooltip=health_tooltip position=StaticTooltipPosition::Bottom>
                    <HorizontalProgressBar
                        class=if is_big { "h-3 sm:h-6" } else { "h-2 sm:h-4" }
                        bar_color="bg-gradient-to-b from-red-500 to-red-700"
                        value=health_percent
                        text=monster_name.clone()
                    />
                </StaticTooltip>
                <CharacterPortrait
                    image_uri=specs.character_specs.portrait.clone()
                    character_name=specs.character_specs.name.clone()
                    just_hurt=just_hurt
                    just_hurt_crit=just_hurt_crit
                    just_blocked=just_blocked
                    is_dead=is_dead
                    statuses=statuses
                />
                <Show when=move || { gold_reward.get() > 0.0 }>
                    <div class="
                    flex gold-float gold-text text-2xl  text-shadow-md
                    absolute left-1/2 top-1/2 transform -translate-x-1/2 -translate-y-1/2 pointer-events-none z-50
                    ">
                        {format!("+{}", format_number(gold_reward.get()))}
                        <img
                            src=img_asset("ui/gold.webp")
                            alt="Gold"
                            class="h-[2em] aspect-square"
                        />
                    </div>
                </Show>
            </div>
            <div class="flex flex-col justify-evenly w-full min-w-16">
                <For
                    each=move || { specs.skill_specs.clone().into_iter().enumerate() }
                    key=|(i, _)| *i
                    let((i, p))
                >
                    <MonsterSkill skill_specs=p index=i monster_index=index />
                </For>
            </div>
        </div>
    }
}

#[component]
fn MonsterSkill(skill_specs: SkillSpecs, index: usize, monster_index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let icon_asset = img_asset(&skill_specs.base.icon);
    let skill_name = skill_specs.base.name.clone();

    let is_dead = Memo::new(move |_| {
        game_context
            .monster_states
            .read()
            .get(monster_index)
            .map(|x| !x.character_state.is_alive)
            .unwrap_or(false)
    });

    let skill_cooldown = Signal::derive(move || {
        if !is_dead.get() {
            (game_context
                .monster_states
                .read()
                .get(monster_index)
                .and_then(|m| m.skill_states.get(index))
                .map(|s| s.elapsed_cooldown)
                .unwrap_or(0.0)
                * 100.0) as f32
        } else {
            0.0
        }
    });

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let show_tooltip = move |_| {
        let skill_specs = Arc::new(skill_specs.clone());
        tooltip_context.set_content(
            move || {
                let skill_specs = skill_specs.clone();
                view! { <SkillTooltip skill_specs=skill_specs /> }.into_any()
            },
            DynamicTooltipPosition::Auto,
        );
    };

    let tooltip_context = expect_context::<DynamicTooltipContext>();
    let hide_tooltip = move |_| tooltip_context.hide();

    let just_triggered = Memo::new(move |_| {
        if !is_dead.get() {
            game_context
                .monster_states
                .read()
                .get(monster_index)
                .and_then(|m| m.skill_states.get(index))
                .map(|s| s.just_triggered)
                .unwrap_or_default()
        } else {
            false
        }
    });

    view! {
        <CircularProgressBar
            bar_width=4
            bar_color="text-amber-700"
            value=skill_cooldown
            reset=just_triggered

            on:mouseenter=show_tooltip
            on:mouseleave=hide_tooltip
        >
            <img
                src=icon_asset
                alt=skill_name
                class="w-full h-full flex-no-shrink fill-current
                drop-shadow-[0px_2px_oklch(13% 0.028 261.692)] invert"
            />
        </CircularProgressBar>
    }
}
