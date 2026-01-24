use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::messages::client::ClientMessage;

use crate::components::{
    events::{EventsContext, Key},
    shared::resources::{GemsCounter, GoldCounter, ShardsCounter},
    ui::{
        buttons::{MenuButton, MenuButtonRed},
        confirm::ConfirmContext,
        fullscreen::FullscreenButton,
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
        move |_| {
            if game_context.quest_rewards.read_untracked().is_some() {
                game_context.open_end_quest.set(true);
            } else {
                (confirm_context.confirm)(
                "Abandoning the Grind will reset the progression, keeping only Items, Gems and Power Shards collected. Are you sure?".into(),
                do_abandon_quest.clone(),
            );
            }
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
        <div class="relative z-50 flex justify-between items-center p-1 xl:p-2
        bg-zinc-800 border-b-1 border-zinc-950 shadow-md/30 h-auto">
            <div class="flex justify-around w-full items-center">
                <GoldCounter value=gold />
                <GemsCounter value=gems />
                <ShardsCounter value=shards />
            </div>
            <div class="flex justify-end space-x-1 xl:space-x-2 w-full">
                <FullscreenButton />
                <WikiButton />
                <MenuButton on:click=move |_| open_inventory()>
                    <span class="inline xl:hidden">"Inv."</span>
                    <span class="hidden xl:inline font-variant:small-caps">"Inventory"</span>
                </MenuButton>
                <MenuButton on:click=move |_| open_passives()>
                    <span class="inline xl:hidden">"Pas."</span>
                    <span class="hidden xl:inline font-variant:small-caps">"Passives"</span>
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
