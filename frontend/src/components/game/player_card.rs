use leptos::html::*;
use leptos::prelude::*;

use shared::data::SkillPrototype;

use crate::components::game::GameContext;
use crate::components::{
    ui::buttons::MainMenuButton,
    ui::progress_bars::{CircularProgressBar, VerticalProgressBar},
};

use super::character::CharacterPortrait;

#[component]
pub fn PlayerCard() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let player_name = move || {
        game_context
            .player_prototype
            .read()
            .character_prototype
            .name
            .clone()
    };

    let health_percent = Signal::derive(move || {
        let max_health = game_context
            .player_prototype
            .read()
            .character_prototype
            .max_health;
        if max_health > 0.0 {
            (game_context.player_state.read().character_state.health / max_health * 100.0) as f32
        } else {
            0.0
        }
    });

    let mana_percent = Signal::derive(move || {
        let max_mana = game_context.player_prototype.read().max_mana;
        if max_mana > 0.0 {
            (game_context.player_state.read().mana / max_mana * 100.0) as f32
        } else {
            0.0
        }
    });

    let is_dead =
        Signal::derive(move || !game_context.player_state.read().character_state.is_alive);

    let just_hurt =
        Signal::derive(move || game_context.player_state.read().character_state.just_hurt);

    view! {
    <div class="flex flex-col gap-2 p-2 bg-zinc-800 rounded-md h-full">
        <div>
            <p class="text-shadow-md shadow-gray-950 text-amber-200 text-xl">
                {player_name}
            </p>
        </div>

        <div class="flex gap-2">
            <VerticalProgressBar class:w-3 class:md:w-6 bar_color="bg-gradient-to-b from-red-500 to-red-700" value=health_percent />
            <CharacterPortrait
                image_asset=game_context.player_prototype.read().character_prototype.portrait.clone()
                character_name="player".to_string()
                just_hurt=just_hurt
                is_dead=is_dead
            />
            <VerticalProgressBar class:w-3 class:md:w-6 bar_color="bg-gradient-to-b from-blue-500 to-blue-700" value=mana_percent />
        </div>

        <div class="grid grid-cols-4 gap-2">
            <For
                each=move || game_context.player_prototype.get().character_prototype.skill_prototypes.into_iter().enumerate()
                key=|(i,_)|  *i
                let((i,p))
            >
                <PlayerSkill prototype=p index=i />
            </For>
        </div>
    </div>
    }
}

#[component]
fn PlayerSkill(prototype: SkillPrototype, index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let skill_cooldown = Signal::derive(move || {
        if prototype.cooldown > 0.0 {
            (game_context
                .player_state
                .read()
                .character_state
                .skill_states
                .get(index)
                .map(|x| x.elapsed_cooldown)
                .unwrap_or_default()
                * 100.0
                / prototype.cooldown) as f32
        } else {
            0.0
        }
    });

    // TODO: Skill component

    let just_triggered = Signal::derive(move || {
        game_context
            .player_state
            .read()
            .character_state
            .skill_states
            .get(index)
            .map(|x| x.just_triggered)
            .unwrap_or_default()
    });

    view! {
        <div class="flex flex-col">
            <CircularProgressBar
                bar_width=4
                bar_color="text-amber-700"
                value=skill_cooldown
                reset=just_triggered
            >
                <img
                    src={format!("./assets/{}",prototype.icon.clone())} alt=prototype.name
                    class="invert drop-shadow-lg w-full h-full flex-no-shrink fill-current"
                />
            </CircularProgressBar>
            <MainMenuButton>
                "auto"
            </MainMenuButton>
        </div>
    }
}
