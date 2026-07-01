use leptos::{html::*, prelude::*};

use crate::components::{
    chat::chat_context::ChatContext,
    events::{EventsContext, Key},
    game::websocket::WebsocketContext,
    shared::resources::{GemsCounter, GoldCounter, ShardsCounter},
    ui::{
        buttons::{MenuButton, MenuButtonRed},
        fullscreen::FullscreenButton,
        header::BaseHeaderMenu,
        wiki::WikiButton,
    },
};
use shared::messages::client::{ClientMessage, TerminateQuestMessage};

use super::GameContext;

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let game_context: GameContext = expect_context();
    let chat_context: ChatContext = expect_context();
    let events_context: EventsContext = expect_context();

    let stop_grind = {
        let conn: WebsocketContext = expect_context();
        move |_| {
            if game_context.area_specs.read_untracked().training {
                conn.send(&ClientMessage::EndQuest);
                conn.send(
                    &TerminateQuestMessage {
                        reward_picks: Default::default(),
                    }
                    .into(),
                );
            } else {
                game_context.open_end_quest.set(true);
            }
        }
    };

    let quit = {
        let navigate = leptos_router::hooks::use_navigate();
        move |_| {
            navigate("/user-dashboard", Default::default());
        }
    };

    let resources = Memo::new(move |_| {
        game_context.player_resources.with(|player_resources| {
            (
                player_resources.gold,
                player_resources.gems,
                player_resources.shards,
                player_resources.passive_points,
            )
        })
    });
    let gold = Signal::derive(move || resources.get().0);
    let gems = Signal::derive(move || resources.get().1);
    let shards = Signal::derive(move || resources.get().2);

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
        <BaseHeaderMenu>
            <div class="flex justify-start space-x-1 xl:space-x-2">
                <FullscreenButton />
                <MenuButton on:click=move |_| {
                    game_context.open_settings.set(!game_context.open_settings.get_untracked())
                }>"⚙"</MenuButton>
                <MenuButton
                    class:hidden
                    class:xl:inline
                    on:click=move |_| {
                        chat_context.opened.set(!chat_context.opened.get_untracked())
                    }
                >
                    "🗪"
                </MenuButton>
                <WikiButton />
            </div>
            <div class="flex-1 flex justify-around w-full items-center">
                <GoldCounter
                    value=gold
                    w_full=true
                    disabled=Signal::derive(move || {
                        !game_context.area_specs.read().can_reward_gold()
                    })
                />
                <GemsCounter
                    value=gems
                    w_full=true
                    disabled=Signal::derive(move || {
                        !game_context.area_specs.read().can_reward_gems()
                    })
                />
                <ShardsCounter
                    value=shards
                    w_full=true
                    disabled=Signal::derive(move || {
                        !game_context.area_specs.read().can_reward_shards()
                    })
                />
            </div>
            <div class="flex justify-end space-x-1 xl:space-x-2">
                <MenuButton on:click=move |_| open_inventory()>
                    <span class="inline xl:hidden">"Inv."</span>
                    <span class="hidden xl:inline font-variant:small-caps">"Inventory"</span>
                </MenuButton>
                <MenuButton on:click=move |_| open_passives()>
                    <span class="inline xl:hidden">"Pas."</span>
                    <span class="hidden xl:inline font-variant:small-caps">"Passives"</span>
                    {move || {
                        let points = resources.get().3;
                        if points > 0 { format!(" ({points})") } else { "".to_string() }
                    }}
                </MenuButton>
                <MenuButton on:click=move |_| open_stats()>"Stats"</MenuButton>
                <MenuButtonRed on:click=stop_grind>"End"</MenuButtonRed>
                <MenuButton on:click=quit>"Back"</MenuButton>
            </div>
        </BaseHeaderMenu>
    }
}
