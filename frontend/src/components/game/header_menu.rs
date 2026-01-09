use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::{computations, constants, messages::client::ClientMessage};

use crate::components::{
    events::{EventsContext, Key},
    settings::SettingsContext,
    shared::resources::{GemsCounter, GoldCounter, ShardsCounter},
    ui::{
        buttons::{MenuButton, MenuButtonRed},
        confirm::ConfirmContext,
        fullscreen::FullscreenButton,
        number::format_number_without_context,
        wiki::WikiButton,
    },
    websocket::WebsocketContext,
};

use super::GameContext;

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let game_context: GameContext = expect_context();
    let events_context: EventsContext = expect_context();

    let do_abandon_quest = Arc::new({
        let conn: WebsocketContext = expect_context();
        move || {
            conn.send(&ClientMessage::EndQuest);
        }
    });

    let try_abandon_quest = {
        let confirm_context: ConfirmContext = expect_context();
        let settings_context: SettingsContext = expect_context();
        move |_| {
            let gold_str = format_number_without_context(
                game_context.player_resources.read().gold_total
                    * computations::exponential(
                        game_context.area_specs.read().item_level_modifier,
                        constants::MONSTER_INCREASE_FACTOR,
                    ),
                settings_context.read_settings().scientific_notation,
            );
            (confirm_context.confirm)(
                format!("Abandoning the Grind will reset the Area Level, Player Level and Gold. You will keep Items, Gems and Power Shards, and collect {} Gold as Temple Donations. Are you sure?",gold_str),
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

    let open_inventory = move || {
        game_context
            .open_inventory
            .set(!game_context.open_inventory.get_untracked());
        game_context.open_statistics.set(false);
        game_context.open_passives.set(false);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('i')) {
            open_inventory()
        }
    });

    let open_passives = move || {
        game_context.open_inventory.set(false);
        game_context
            .open_passives
            .set(!game_context.open_passives.get_untracked());
        game_context.open_statistics.set(false);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('p')) {
            open_passives()
        }
    });

    let open_stats = move || {
        game_context.open_inventory.set(false);
        game_context.open_passives.set(false);
        game_context
            .open_statistics
            .set(!game_context.open_statistics.get_untracked());
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('s')) {
            open_stats()
        }
    });

    view! {
        <div class="relative z-50 flex justify-between items-center p-1 xl:p-2 bg-zinc-800 shadow-md h-auto">
            <div class="flex justify-around w-full items-center">
                <GoldCounter value=gold />
                <GemsCounter value=gems />
                <ShardsCounter value=shards />
            </div>
            <div class="flex justify-end space-x-1 xl:space-x-2 w-full">
                <FullscreenButton />
                <WikiButton />
                <MenuButton on:click=move |_| open_inventory()>"Inventory"</MenuButton>
                <MenuButton on:click=move |_| open_passives()>
                    "Passives"
                    {move || {
                        let points = game_context.player_resources.read().passive_points;
                        if points > 0 { format!(" ({points})") } else { "".to_string() }
                    }}
                </MenuButton>
                <MenuButton on:click=move |_| open_stats()>"Stats"</MenuButton>
                <MenuButtonRed on:click=try_abandon_quest>"Stop"</MenuButtonRed>
                <MenuButton on:click=quit>"Back"</MenuButton>
            </div>
        </div>
    }
}
