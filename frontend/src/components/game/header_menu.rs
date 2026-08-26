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
    let show_power_shard_tip = RwSignal::new(false);
    let previous_max_power_shard_level = RwSignal::new(
        game_context
            .area_state
            .read_untracked()
            .max_power_shard_level_ever,
    );

    Effect::new(move || {
        let max_power_shard_level = game_context.area_state.read().max_power_shard_level_ever;
        let previous_level = previous_max_power_shard_level.get_untracked();

        if game_context
            .area_id
            .with(|area_id| area_id == "inn_basement.json")
            && previous_level == 0
            && max_power_shard_level > 0
        {
            show_power_shard_tip.set(true);
        }

        previous_max_power_shard_level.set(max_power_shard_level);
    });

    let stop_grind = {
        let conn: WebsocketContext = expect_context();
        move |_| {
            show_power_shard_tip.set(false);
            if game_context.area_specs.read_untracked().training {
                conn.send(&ClientMessage::EndQuest);
                conn.send(
                    &TerminateQuestMessage {
                        reward_picks: Default::default(),
                    }
                    .into(),
                );
            } else {
                game_context.open_end_grind.set(true);
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
    let shard_level_exceeded = Signal::derive(move || {
        let area_specs = game_context.area_specs.read();
        area_specs.can_reward_shards()
            && game_context.area_state.read().area_level > area_specs.max_power_shard_level
    });

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
                            || shard_level_exceeded.get()
                    })
                    disabled_description=Signal::derive(move || {
                        shard_level_exceeded
                            .get()
                            .then(|| {
                                "No more Power Shards can be unlocked during this Grind."
                                    .to_string()
                            })
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
                <div class="relative">
                    <MenuButtonRed on:click=stop_grind>"End"</MenuButtonRed>
                    <Show when=move || show_power_shard_tip.get()>
                        <div
                            role="status"
                            class="pointer-events-none absolute right-0 top-full z-50 mt-3 w-72 max-w-[calc(100vw-1rem)] rounded border border-amber-300/80 bg-zinc-900 px-3 py-2 text-left text-xs font-normal normal-case tracking-normal text-zinc-100 shadow-xl xl:text-sm"
                        >
                            <span class="font-bold text-amber-300">"Tip: "</span>
                            "Click here to end the Grind and return to Town, where you can spend your Power Shard by Ascending a Passive and unlock a new Skill Slot in the Temple."
                            <span class="absolute -top-2 right-4 h-0 w-0 border-x-8 border-b-8 border-x-transparent border-b-amber-300/80" />
                        </div>
                    </Show>
                </div>
                <MenuButton on:click=quit>"Back"</MenuButton>
            </div>
        </BaseHeaderMenu>
    }
}
