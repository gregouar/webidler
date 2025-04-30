use leptos::html::*;
use leptos::prelude::*;

use shared::data::skill::SkillSpecs;
use shared::messages::client::LevelUpSkillMessage;
use shared::messages::client::SetAutoSkillMessage;
use shared::messages::client::UseSkillMessage;

use crate::assets::img_asset;
use crate::components::{
    ui::{
        buttons::{FancyButton, Toggle},
        number::format_number,
        progress_bars::{CircularProgressBar, HorizontalProgressBar, VerticalProgressBar},
        tooltip::StaticTooltip,
    },
    websocket::WebsocketContext,
};

use super::character::CharacterPortrait;
use super::GameContext;

#[component]
pub fn PlayerCard() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let health_percent = Signal::derive(move || {
        let max_health = game_context.player_specs.read().character_specs.max_health;
        if max_health > 0.0 {
            (game_context.player_state.read().character_state.health / max_health * 100.0) as f32
        } else {
            0.0
        }
    });

    let mana_percent = Signal::derive(move || {
        let max_mana = game_context.player_specs.read().max_mana;
        if max_mana > 0.0 {
            (game_context.player_state.read().mana / max_mana * 100.0) as f32
        } else {
            0.0
        }
    });

    let xp_percent = Signal::derive(move || {
        let max_xp = game_context.player_specs.read().experience_needed;
        if max_xp > 0.0 {
            (game_context.player_state.read().experience / max_xp * 100.0) as f32
        } else {
            0.0
        }
    });

    let is_dead =
        Signal::derive(move || !game_context.player_state.read().character_state.is_alive);

    let just_hurt =
        Signal::derive(move || game_context.player_state.read().character_state.just_hurt);

    view! {
        <style>
            "
            @keyframes player-fade-in {
             0% { transform: translateX(-100%); opacity: 0; }
             65% { transform: translateX(0%); opacity: 1; }
             80% { transform: translateX(-5%); }
             100% { transform: translateX(0%); }
            }
            
            @keyframes player-fade-out {
             from { opacity: 1; transform: translateY(0%); }
             to { opacity: 0; transform: translateY(100%); }
            }
            "
        </style>
        // <div class="overflow-hidden">
        <div class="w-full flex flex-col gap-2 p-2 bg-zinc-800 rounded-md h-full shadow-md ring-1 ring-zinc-950">

            // style=move || {
            // if is_dead.get() {
            // "animation: player-fade-out 3s ease-in; animation-fill-mode: both;"
            // } else {
            // "animation: player-fade-in 1s ease-out; animation-fill-mode: both;"
            // }
            // }
            <div>
                <PlayerName />
            </div>

            <div class="flex flex-col gap-2">
                <div class="flex gap-2">
                    <VerticalProgressBar
                        class:w-3
                        class:md:w-6
                        bar_color="bg-gradient-to-l from-red-500 to-red-700"
                        value=health_percent
                    />
                    <CharacterPortrait
                        image_uri=game_context.player_specs.read().character_specs.portrait.clone()
                        character_name="player".to_string()
                        just_hurt=just_hurt
                        is_dead=is_dead
                    />
                    <VerticalProgressBar
                        class:w-3
                        class:md:w-6
                        bar_color="bg-gradient-to-l from-blue-500 to-blue-700"
                        value=mana_percent
                    />
                </div>
                <HorizontalProgressBar
                    class:h-2
                    class:sm:h-4
                    bar_color="bg-gradient-to-b from-neutral-300 to-neutral-500"
                    // TODO: XP
                    value=xp_percent
                />
            </div>

            <div class="grid grid-cols-4 gap-2">
                <For
                    each=move || {
                        game_context.player_specs.get().skill_specs.into_iter().enumerate()
                    }
                    key=|(i, _)| *i
                    let((i, p))
                >
                    <PlayerSkill specs=p index=i />
                </For>
            </div>
        </div>
    }
}

#[component]
pub fn PlayerName() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let player_name = move || {
        game_context
            .player_specs
            .read()
            .character_specs
            .name
            .clone()
    };

    view! {
        <p class="text-shadow-md shadow-gray-950 text-amber-200 text-xl">
            <span class="font-bold">{player_name}</span>
            {move || format!(" — Level: {}", game_context.player_specs.read().level)}
        </p>
    }
}

#[component]
fn PlayerSkill(specs: SkillSpecs, index: usize) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let skill_cooldown = Signal::derive(move || {
        if specs.cooldown > 0.0 {
            (game_context
                .player_state
                .read()
                .skill_states
                .get(index)
                .map(|x| x.elapsed_cooldown)
                .unwrap_or_default()
                * 100.0
                / specs.cooldown) as f32
        } else {
            0.0
        }
    });

    let initial_auto_use = *game_context
        .player_specs
        .get()
        .auto_skills
        .get(index)
        .unwrap_or(&false);

    let just_triggered = Signal::derive(move || {
        game_context
            .player_state
            .read()
            .skill_states
            .get(index)
            .map(|x| x.just_triggered)
            .unwrap_or_default()
    });

    let conn = expect_context::<WebsocketContext>();
    let use_skill = move |_| {
        // TODO: Add constraint/limit rates?
        conn.send(
            &UseSkillMessage {
                skill_index: index as u8,
            }
            .into(),
        );
    };

    let conn = expect_context::<WebsocketContext>();
    let set_auto_skill = move |value| {
        // TODO: Add constraint/limit rates?
        conn.send(
            &SetAutoSkillMessage {
                skill_index: index as u8,
                auto_use: value,
            }
            .into(),
        );
    };

    let conn = expect_context::<WebsocketContext>();
    let level_up = move |_| {
        // TODO: Add constraint/limit rates?
        conn.send(
            &LevelUpSkillMessage {
                skill_index: index as u8,
            }
            .into(),
        );
    };

    let level_up_cost = Signal::derive(move || {
        game_context
            .player_state
            .read()
            .skill_states
            .get(index)
            .map(|x| x.next_upgrade_cost)
            .unwrap_or_default()
    });

    let disable_level_up =
        Signal::derive(move || level_up_cost.get() > game_context.player_resources.read().gold);

    let cost_tooltip =
        Signal::derive(move || format!("{} Gold", format_number(level_up_cost.get())));

    view! {
        <div class="flex flex-col">
            <CircularProgressBar
                bar_width=4
                bar_color="text-amber-700"
                value=skill_cooldown
                reset=just_triggered
            >
                <button class="active:brightness-50 active:sepia p-1" on:click=use_skill>
                    <img
                        src=img_asset(&specs.icon)
                        alt=specs.name
                        class="w-full h-full flex-no-shrink fill-current
                        drop-shadow-[0px_4px_oklch(13% 0.028 261.692)] invert"
                    />
                </button>
            </CircularProgressBar>

            <div class="flex justify-around">
                <Toggle
                    toggle_callback=set_auto_skill
                    initial=initial_auto_use
                    label="auto".to_string()
                />
                <StaticTooltip tooltip=cost_tooltip>
                    <FancyButton disabled=disable_level_up on:click=level_up>
                        <span class="text-2xl">"+"</span>
                    </FancyButton>
                </StaticTooltip>

            </div>
        </div>
    }
}
