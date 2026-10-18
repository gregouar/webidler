use leptos::{html::*, prelude::*};

use crate::components::{
    chat::chat_context::ChatContext,
    events::{EventsContext, Key},
    game::websocket::WebsocketContext,
    icons::header_icons::{AchievementsIcon, InventoryIcon, PassivesIcon, PetsIcon, StatsIcon},
    shared::resources::{GemsCounter, GoldCounter, ShardsCounter},
    ui::{
        buttons::{MenuButton, MenuButtonRed},
        fullscreen::FullscreenButton,
        header::BaseHeaderMenu,
        tutorial_popup::{TutorialPopup, TutorialPopupPosition},
        wiki::WikiButton,
    },
};
use shared::messages::client::{ClientMessage, TerminateGrindMessage};

use super::GameContext;

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let game_context: GameContext = expect_context();
    let chat_context: ChatContext = expect_context();
    let events_context: EventsContext = expect_context();
    let show_passive_point_tutorial = Signal::derive(move || {
        game_context
            .area_id
            .with(|area_id| area_id == "inn_basement.json")
            && game_context.area_state.read().max_area_level_ever < 10
            && game_context
                .passives_tree_state
                .read()
                .purchased_nodes
                .is_empty()
            && game_context.player_resources.read().passive_points > 0
            && !game_context.open_passives.get()
    });
    let power_shard_tip_triggered = RwSignal::new(false);
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
            && !show_passive_point_tutorial.get()
        {
            power_shard_tip_triggered.set(true);
        }

        previous_max_power_shard_level.set(max_power_shard_level);
    });

    let switch_panel = move |open_signal: RwSignal<bool>| {
        let open_value = !open_signal.get_untracked();

        game_context.open_inventory.set(false);
        game_context.open_passives.set(false);
        game_context.open_statistics.set(false);
        game_context.open_skills.set(false);
        game_context.open_end_grind.set(false);
        game_context.open_achievements.set(false);
        game_context.open_pets.set(false);

        open_signal.set(open_value);
    };

    let stop_grind = {
        let conn: WebsocketContext = expect_context();
        move |_| {
            if game_context.area_specs.read_untracked().training {
                conn.send(&ClientMessage::EndGrind);
                conn.send(
                    &TerminateGrindMessage {
                        reward_picks: Default::default(),
                        quest_reward_picked: Default::default(),
                    }
                    .into(),
                );
            } else {
                switch_panel(game_context.open_end_grind);
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
        switch_panel(game_context.open_inventory);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('i')) {
            open_inventory()
        }
    });

    let open_passives = move || {
        switch_panel(game_context.open_passives);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('p')) {
            open_passives()
        }
    });

    let open_stats = move || {
        switch_panel(game_context.open_statistics);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('s')) {
            open_stats()
        }
    });

    let open_pets = move || {
        switch_panel(game_context.open_pets);
    };

    let open_achievements = move || {
        switch_panel(game_context.open_achievements);
    };

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
                <MenuButton on:click=move |_| open_inventory() title="Inventory">
                    <InventoryIcon />
                </MenuButton>
                <TutorialPopup
                    show=show_passive_point_tutorial
                    position=TutorialPopupPosition::BelowRight
                    message="Click here to spend your Passive Point on a node in your current Grind."
                >
                    <MenuButton on:click=move |_| open_passives() title="Passives">
                        <PassivesIcon />
                        {move || {
                            let points = resources.get().3;
                            if points > 0 { format!(" ({points})") } else { "".to_string() }
                        }}
                    </MenuButton>
                </TutorialPopup>
                <MenuButton on:click=move |_| open_stats() title="Stats">
                    <StatsIcon />
                </MenuButton>
                <MenuButton
                    on:click=move |_| { open_pets() }
                    disabled=move || game_context.user_unlocks.read().pets.is_empty()
                    title="Pets"
                >
                    <PetsIcon />
                </MenuButton>
                <MenuButton on:click=move |_| { open_achievements() } title="Achievements">
                    <AchievementsIcon />
                </MenuButton>
                <TutorialPopup
                    show=power_shard_tip_triggered
                    position=TutorialPopupPosition::BelowRight
                    message="Click here to end the Grind and return to Town, where you can spend your Power Shard by Ascending a Passive and unlock a new Skill Slot in the Temple."
                >
                    <MenuButtonRed on:click=stop_grind>"End"</MenuButtonRed>
                </TutorialPopup>
                <MenuButton on:click=quit>"Back"</MenuButton>
            </div>
        </BaseHeaderMenu>
    }
}
