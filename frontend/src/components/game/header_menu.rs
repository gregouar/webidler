use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::{computations, constants, messages::client::ClientMessage};

use crate::components::{
    shared::resources::{GemsCounter, GoldCounter, ShardsCounter},
    ui::{
        buttons::{MenuButton, MenuButtonRed},
        confirm::ConfirmContext,
        fullscreen::FullscreenButton,
        number::format_number,
    },
    websocket::WebsocketContext,
};

use super::GameContext;

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let do_abandon_quest = Arc::new({
        let conn = expect_context::<WebsocketContext>();
        move || {
            conn.send(&ClientMessage::EndQuest);
        }
    });

    let try_abandon_quest = {
        let confirm_context = expect_context::<ConfirmContext>();
        move |_| {
            (confirm_context.confirm)(
                format!("Abandoning the Grind will reset the Area Level, Player Level and Gold. You will keep Items, Gems and Power Shards, and collect {} Gold as Temple Donations. Are you sure?",format_number(
                                        game_context.player_resources.read().gold_total
                                            * computations::exponential(
                                                game_context.area_specs.read().item_level_modifier,
                                                constants::MONSTER_INCREASE_FACTOR,
                                            ),
                                    )),
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

    let gold = Signal::derive(move || game_context.player_resources.read().gold);
    let gems = Signal::derive(move || game_context.player_resources.read().gems);
    let shards = Signal::derive(move || game_context.player_resources.read().shards);

    view! {
        <div class="relative z-50 flex justify-between items-center p-1 xl:p-2 bg-zinc-800 shadow-md h-auto">
            <div class="flex justify-around w-full items-center">
                <GoldCounter value=gold />
                <GemsCounter value=gems />
                <ShardsCounter value=shards />
            </div>
            <div class="flex justify-end space-x-1 xl:space-x-2 w-full">
                <FullscreenButton />
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
                <MenuButtonRed on:click=try_abandon_quest>"Stop"</MenuButtonRed>
                <MenuButton on:click=quit>"Quit"</MenuButton>
            </div>
        </div>
    }
}
