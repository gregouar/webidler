use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::messages::client::ClientMessage;

use crate::{
    assets::{img_asset, music_asset},
    components::{
        ui::{
            buttons::{MenuButton, MenuButtonRed},
            confirm::ConfirmContext,
            number::Number,
            tooltip::{StaticTooltip, StaticTooltipPosition},
        },
        websocket::WebsocketContext,
    },
};

use super::GameContext;

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let do_abandon_quest = Arc::new({
        let conn = expect_context::<WebsocketContext>();
        let navigate = leptos_router::hooks::use_navigate();
        move || {
            conn.send(&ClientMessage::EndQuest);
            // TODO: Bring to summary page...
            navigate("/town", Default::default());
        }
    });

    let try_abandon_quest = {
        let confirm_context = expect_context::<ConfirmContext>();
        move |_| {
            (confirm_context.confirm)(
                "Abandoning the grind will reset the area level, player level and gold, you will only keep items, gems and power shards. Are you sure?".to_string(),
                do_abandon_quest.clone(),
            );
        }
    };

    let quit = {
        let navigate = leptos_router::hooks::use_navigate();
        move |_| {
            navigate("/user-dashboard", Default::default());
        }
    };

    let musics = {
        let game_context = expect_context::<GameContext>();
        move || {
            game_context
                .area_specs
                .read()
                .musics
                .iter()
                .map(|m| music_asset(m))
                .collect::<Vec<_>>()
        }
    };

    let gold = {
        let game_context = expect_context::<GameContext>();
        Signal::derive(move || game_context.player_resources.read().gold)
    };
    let gems = {
        let game_context = expect_context::<GameContext>();
        Signal::derive(move || game_context.player_resources.read().gems)
    };
    let shards = {
        let game_context = expect_context::<GameContext>();
        Signal::derive(move || game_context.player_resources.read().shards)
    };

    let game_context = expect_context::<GameContext>();
    view! {
        <div class="relative z-50 flex justify-between items-center p-2 bg-zinc-800 shadow-md h-auto">
            <div class="flex justify-around w-full items-center">
                <ResourceCounter
                    class:text-amber-200
                    icon="ui/gold.webp"
                    name="Gold"
                    description="To buy level up for skills. Reset at every grind."
                    value=gold
                />
                <ResourceCounter
                    class:text-violet-200
                    icon="ui/gems.webp"
                    name="Gems"
                    description="To buy items in the market between grinds."
                    value=gems
                />
                <ResourceCounter
                    class:text-cyan-200
                    icon="ui/power_shard.webp"
                    name="Power Shards"
                    description="To permanently increase power of passive skills."
                    value=shards
                />
            </div>
            <div class="flex justify-end space-x-2  w-full">
                <audio autoplay loop controls>
                    {move || {
                        musics()
                            .into_iter()
                            .map(|src| {
                                view! { <source src=src /> }
                            })
                            .collect_view()
                    }}
                </audio>

                <MenuButton on:click=move |_| {
                    game_context.open_inventory.set(!game_context.open_inventory.get());
                    game_context.open_statistics.set(false);
                    game_context.open_passives.set(false);
                }>"Inventory"</MenuButton>
                <MenuButton on:click=move |_| {
                    game_context.open_inventory.set(false);
                    game_context.open_passives.set(!game_context.open_passives.get());
                    game_context.open_statistics.set(false);
                }>
                    "Passives"
                    {move || {
                        let points = game_context.player_resources.read().passive_points;
                        if points > 0 { format!(" ({points})") } else { "".to_string() }
                    }}
                </MenuButton>
                <MenuButton on:click=move |_| {
                    game_context.open_inventory.set(false);
                    game_context.open_passives.set(false);
                    game_context.open_statistics.set(!game_context.open_statistics.get());
                }>"Stats"</MenuButton>
                <MenuButtonRed on:click=try_abandon_quest>"Stop Grind"</MenuButtonRed>
                <MenuButton on:click=quit>"Quit"</MenuButton>
            </div>
        </div>
    }
}

#[component]
fn ResourceCounter(
    icon: &'static str,
    name: &'static str,
    description: &'static str,
    value: Signal<f64>,
) -> impl IntoView {
    let tooltip = move || {
        view! {
            <div class="flex flex-col space-y-1">
                <div class="font-semibold text-white">{name}</div>
                <div class="text-sm text-zinc-300 max-w-xs">{description}</div>
            </div>
        }
    };
    view! {
        <div class="flex-1 text-shadow-md shadow-gray-950 text-xl flex justify-center items-center space-x-1">
            <div class="font-mono tabular-nums w-[8ch] text-right">
                <Number value=value />
            </div>
            <StaticTooltip tooltip=tooltip position=StaticTooltipPosition::Bottom>
                <img src=img_asset(icon) alt=name class="h-[2em] aspect-square" />
            </StaticTooltip>
        </div>
    }
}
