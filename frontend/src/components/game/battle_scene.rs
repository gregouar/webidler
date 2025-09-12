use leptos::{html::*, prelude::*};

use shared::constants::WAVES_PER_AREA_LEVEL;
use shared::messages::client::{GoBackLevelMessage, SetAutoProgressMessage};

use crate::assets::img_asset;
use crate::components::websocket::WebsocketContext;

use super::loot_queue::LootQueue;
use super::monsters_grid::MonstersGrid;
use super::player_card::PlayerCard;
use super::GameContext;

#[component]
pub fn BattleScene() -> impl IntoView {
    view! {
        <div class="absolute inset-0 p-2 xl:p-4">
            <div class="relative w-full max-h-full flex justify-between gap-2 xl:gap-4 ">
                <PlayerCard />
                <div class="w-2/3 aspect-[12/8] flex flex-col shadow-lg rounded-md overflow-hidden">
                    <BattleSceneHeader />
                    <MonstersGrid />
                    <LootQueue />
                    <BattleSceneFooter />
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn BattleSceneHeader() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let auto_icon = move || {
        if game_context.area_state.read().auto_progress {
            "⏸"
        } else {
            "▶"
        }
    };

    let go_back = {
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            conn.send(&GoBackLevelMessage { amount: 1 }.into());
        }
    };

    let toggle_auto_progress = {
        let conn = expect_context::<WebsocketContext>();
        move |_| {
            let auto_progress = !game_context.area_state.read_untracked().auto_progress;
            game_context.area_state.write().auto_progress = auto_progress;
            conn.send(
                &SetAutoProgressMessage {
                    value: auto_progress,
                }
                .into(),
            );
        }
    };

    let header_background = move || {
        format!(
            "background-image: url('{}');",
            img_asset(&game_context.area_specs.read().header_background)
        )
    };

    view! {
        <div
            class="h-8 xl:h-16 relative overflow-hidden w-full
            bg-center bg-repeat-x flex items-center justify-between px-4"
            style=header_background
        >
            <div class="absolute inset-0 bg-gradient-to-r from-transparent via-zinc-950 to-transparent blur-lg"></div>

            <div class="w-12 flex justify-start">
                <button
                    class="btn text-2xl xl:text-4xl text-amber-300 font-bold drop-shadow-[0_0_6px_rgba(0,0,10,0.8)]
                    hover:text-amber-400 hover:drop-shadow-[0_0_8px_rgba(255,200,50,1)] 
                    active:scale-90 active:brightness-125 transition"
                    title="Go Back One Level"
                    on:click=go_back
                >
                    "←"
                </button>
            </div>

            <div class="flex-1 text-center">
                <p class="relative z-10 text-shadow text-amber-200 text-lg xl:text-2xl font-bold">
                    <span class="[font-variant:small-caps]">
                        {move || game_context.area_specs.read().name.clone()}
                    </span>
                    {move || {
                        format!(" — Area Level: {}", game_context.area_state.read().area_level)
                    }}
                </p>
            </div>

            <div class="w-12 flex justify-end">
                <button
                    class="btn text-xl xl:text-3xl text-amber-300 font-bold drop-shadow-[0_0_6px_rgba(0,0,10,0.8)]
                    hover:text-amber-400 hover:drop-shadow-[0_0_8px_rgba(255,200,50,1)] 
                    active:scale-90 active:brightness-125 transition"
                    title="Toggle Auto Progress"
                    on:click=toggle_auto_progress
                >
                    {auto_icon}
                </button>
            </div>
        </div>
    }
}

#[component]
pub fn BattleSceneFooter() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let footer_background = move || {
        format!(
            "background-image: url('{}');",
            img_asset(&game_context.area_specs.read().footer_background)
        )
    };

    view! {
        // h-8 xl:h-16
        <div
            class="h-8 xl:h-16 relative overflow-hidden z-10 w-full
            bg-center bg-repeat-x flex items-center justify-center"
            style=footer_background
        >
            <div class="absolute inset-0 bg-gradient-to-r from-transparent via-zinc-950 to-transparent blur-lg"></div>
            <p class="relative text-shadow-sm shadow-gray-950 text-amber-200 text-base xl:text-2xl font-bold">
                {move || {
                    if game_context.area_state.read().is_boss {
                        "Boss".to_string()
                    } else {
                        format!(
                            "Wave: {}/{}",
                            game_context.area_state.read().waves_done,
                            WAVES_PER_AREA_LEVEL,
                        )
                    }
                }}
            </p>
        </div>
    }
}
