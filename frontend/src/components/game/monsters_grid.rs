use leptos::html::*;
use leptos::prelude::*;

use rand::Rng;

use shared::data::MonsterSpecs;
use shared::data::SkillSpecs;

use crate::assets::img_asset;
use crate::components::ui::progress_bars::{CircularProgressBar, HorizontalProgressBar};

use super::character::CharacterPortrait;
use super::GameContext;

#[component]
pub fn MonstersGrid() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let all_monsters_dead = Signal::derive(move || {
        game_context
            .monster_states
            .read()
            .iter()
            .all(|x| !x.character_state.is_alive)
    });

    let header_background = "bg-[url(./assets/images/worlds/forest_header.webp)]";
    let footer_background = "bg-[url(./assets/images/worlds/forest_footer.webp)]";

    // TODO: double buffering to allow in and out at the same time
    view! {
        <div class="shadow-lg rounded-md overflow-hidden  w-full">
            <div class=format!(
                "{header_background} relative overflow-hidden w-full h-16 bg-center bg-repeat-x flex items-center justify-center",
            )>
                <div class="absolute inset-0 bg-gradient-to-r from-transparent via-zinc-950 to-transparent blur-lg"></div>
                <p class="relative text-shadow-sm shadow-gray-950 text-amber-200 text-2xl font-bold">
                    <span class="[font-variant:small-caps]">"The Forest"</span>
                    " — Area Level: 1"
                </p>
            </div>
            <div class="
            grid grid-cols-3 grid-rows-2 p-2 gap-2 grid-flow-col items-center
            h-full overflow-hidden shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]
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
                        view! {
                            <div style=move || {
                                if all_monsters_dead.get() {
                                    format!(
                                        "animation: monster-fade-out 1s ease-in; animation-fill-mode: both;",
                                    )
                                } else {
                                    format!(
                                        "animation: monster-fade-in 1s ease-out; animation-fill-mode: both; {}",
                                        animation_delay,
                                    )
                                }
                            }>
                                <MonsterCard specs=specs index=index />
                            </div>
                        }
                    }
                />
            </div>
            <div class=format!("{footer_background} w-full h-16 bg-center bg-repeat-x")></div>
        </div>
    }
}

#[component]
fn MonsterCard(specs: MonsterSpecs, index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let health_percent = Signal::derive(move || {
        let max_health = specs.character_specs.max_health;
        if max_health > 0.0 {
            (game_context
                .monster_states
                .read()
                .get(index)
                .map(|s| s.character_state.health)
                .unwrap_or_default()
                / specs.character_specs.max_health
                * 100.0) as f32
        } else {
            0.0
        }
    });

    let is_dead = Signal::derive(move || {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|x| !x.character_state.is_alive)
            .unwrap_or(false)
    });

    let just_hurt = Signal::derive(move || {
        game_context
            .monster_states
            .read()
            .get(index)
            .map(|x| x.character_state.just_hurt)
            .unwrap_or(false)
    });

    view! {
        <div class="grid grid-cols-4 w-full bg-zinc-800 shadow-md rounded-md gap-2 p-2 ring-1 ring-zinc-950">
            <div class="flex flex-col gap-2 col-span-3">
                <HorizontalProgressBar
                    class:h-2
                    class:sm:h-4
                    bar_color="bg-gradient-to-b from-red-500 to-red-700"
                    value=health_percent
                    text=specs.character_specs.name.clone()
                />
                <CharacterPortrait
                    image_uri=specs.character_specs.portrait.clone()
                    character_name=specs.character_specs.name.clone()
                    just_hurt=just_hurt
                    is_dead=is_dead
                />
            </div>
            <div class="flex flex-col justify-evenly w-full min-w-16">
                <For
                    each=move || {
                        specs.character_specs.skill_specs.clone().into_iter().enumerate()
                    }
                    key=|(i, _)| *i
                    let((i, p))
                >
                    <MonsterSkill specs=p index=i monster_index=index />
                </For>
            </div>
        </div>
    }
}

#[component]
fn MonsterSkill(specs: SkillSpecs, index: usize, monster_index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let is_dead = move || {
        game_context
            .monster_states
            .read()
            .get(monster_index)
            .map(|x| !x.character_state.is_alive)
            .unwrap_or(false)
    };

    let skill_cooldown = Signal::derive(move || {
        if !is_dead() && specs.cooldown > 0.0 {
            (game_context
                .monster_states
                .read()
                .get(monster_index)
                .map(|m| m.character_state.skill_states.get(index))
                .flatten()
                .map(|s| s.elapsed_cooldown)
                .unwrap_or(0.0)
                * 100.0
                / specs.cooldown) as f32
        } else {
            0.0
        }
    });

    let just_triggered = Signal::derive(move || {
        if !is_dead() {
            game_context
                .monster_states
                .read()
                .get(monster_index)
                .map(|m| m.character_state.skill_states.get(index))
                .flatten()
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
        >
            <img
                src=img_asset(&specs.icon)
                alt=specs.name
                class="w-full h-full flex-no-shrink fill-current
                drop-shadow-[0px_2px_oklch(13% 0.028 261.692)] invert"
            />
        </CircularProgressBar>
    }
}
