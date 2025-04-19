use leptos::html::*;
use leptos::prelude::*;

use rand::Rng;

use shared::data::MonsterPrototype;
use shared::data::SkillPrototype;

use crate::components::ui::progress_bars::{CircularProgressBar, HorizontalProgressBar};

use super::GameContext;
use super::character::CharacterPortrait;

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

    // TODO: double buffering to allow in and out at the same time
    view! {
        <div class="">
            <div class="w-full h-16 bg-[url(./assets/worlds/forest_header.webp)] bg-center bg-repeat-x ">
            </div>
            <div class="grid grid-cols-3 grid-rows-2 mt-2 mb-2 gap-2 grid-flow-col items-center h-full overflow-hidden">
                <style>"
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
                "</style>
                <For
                    each=move || game_context.monster_prototypes.get().into_iter().enumerate()
                    // We need a unique key to replace old elements
                    key=move |(index,_)| (game_context.monster_wave.get(), *index)
                    children=move |(index, prototype)| {
                        let animation_delay = format!("animation-delay: {}s;", rand::rng().random_range(0.0..=0.2f32));
                        view! {
                            <div
                                style=move|| if all_monsters_dead.get()
                                    {format!("animation: monster-fade-out 1s ease-in; animation-fill-mode: both;")}
                                    else {format!("animation: monster-fade-in 1s ease-out; animation-fill-mode: both; {}",animation_delay)}
                            >
                                <MonsterCard prototype=prototype index=index />
                            </div>
                        }
                    }
                />
            </div>
            <div class="w-full h-16 bg-[url(./assets/worlds/forest_footer.webp)] bg-center bg-repeat-x ">
            </div>
        </div>
    }
}

#[component]
fn MonsterCard(prototype: MonsterPrototype, index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let health_percent = Signal::derive(move || {
        let max_health = prototype.character_prototype.max_health;
        if max_health > 0.0 {
            (game_context
                .monster_states
                .read()
                .get(index)
                .map(|s| s.character_state.health)
                .unwrap_or_default()
                / prototype.character_prototype.max_health
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
        <div class="grid grid-cols-4 w-full bg-zinc-800 rounded-md gap-2 p-2">
            <div class="flex flex-col gap-2 col-span-3">
                <HorizontalProgressBar
                    class:h-2 class:sm:h-4 bar_color="bg-gradient-to-b from-red-500 to-red-700"
                    value=health_percent text=prototype.character_prototype.name.clone()
                />
                <CharacterPortrait
                    image_asset=prototype.character_prototype.portrait.clone()
                    character_name=prototype.character_prototype.name.clone()
                    just_hurt=just_hurt
                    is_dead=is_dead
                />
            </div>
            <div class="flex flex-col justify-evenly w-full min-w-16">
                <For
                    each=move || prototype.character_prototype.skill_prototypes.clone().into_iter().enumerate()
                    key=|(i,_)|  *i
                    let((i,p))
                >
                    <MonsterSkill prototype=p index=i monster_index=index />
                </For>
            </div>
        </div>
    }
}

#[component]
fn MonsterSkill(prototype: SkillPrototype, index: usize, monster_index: usize) -> impl IntoView {
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
        if !is_dead() && prototype.cooldown > 0.0 {
            (game_context
                .monster_states
                .read()
                .get(monster_index)
                .map(|m| m.character_state.skill_states.get(index))
                .flatten()
                .map(|s| s.elapsed_cooldown)
                .unwrap_or(0.0)
                * 100.0
                / prototype.cooldown) as f32
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
            bar_width=4 bar_color="text-amber-700"
            value=skill_cooldown
            reset=just_triggered
        >
            <img
                src={format!("./assets/{}",prototype.icon.clone())} alt=prototype.name
                class="invert drop-shadow-lg w-full h-full flex-no-shrink fill-current"
            />
        </CircularProgressBar>
    }
}
