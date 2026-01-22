use std::sync::Arc;

use leptos::{html::*, prelude::*};

use crate::components::{
    game::GameContext,
    shared::{
        item_card::ItemCard,
        resources::{GemsCounter, GoldCounter, ShardsCounter},
    },
    ui::{
        buttons::{CloseButton, MenuButton},
        confirm::ConfirmContext,
        menu_panel::{MenuPanel, PanelTitle},
        number::format_duration,
    },
    websocket::WebsocketContext,
};
use shared::{computations, constants, messages::client::TerminateQuestMessage};

#[component]
pub fn EndQuestPanel() -> impl IntoView {
    let game_context: GameContext = expect_context();

    let open = game_context.open_end_quest;

    Effect::new(move || {
        if game_context.quest_rewards.read().is_some() {
            open.set(true);
        }
    });

    view! {
        <MenuPanel open w_full=false>
            <EndQuest open />
        </MenuPanel>
    }
}

#[component]
fn EndQuest(open: RwSignal<bool>) -> impl IntoView {
    let game_context: GameContext = expect_context();

    let stats = move || game_context.game_stats.read();

    let gold_donation_value = Signal::derive(move || {
        game_context.player_resources.read().gold_total
            * computations::exponential(
                game_context.area_specs.read().item_level_modifier,
                constants::MONSTER_INCREASE_FACTOR,
            )
    });

    let gems_value = Signal::derive(move || game_context.player_resources.read().gems);
    let shards_value = Signal::derive(move || game_context.player_resources.read().shards);

    let area_completed = move || {
        game_context
            .area_state
            .read()
            .max_area_level
            .saturating_sub(game_context.area_specs.read().starting_level)
            + 1
    };

    let item_reward_picked = RwSignal::new(None);

    let do_confirm_end = Arc::new({
        let conn: WebsocketContext = expect_context();
        move || {
            conn.send(
                &TerminateQuestMessage {
                    item_index: item_reward_picked.get_untracked().map(|x| x as u8),
                }
                .into(),
            );
        }
    });

    let try_confirm_end = {
        let confirm_context: ConfirmContext = expect_context();
        move |_| {
            if item_reward_picked.read_untracked().is_some() {
                do_confirm_end.clone()();
            } else {
                (confirm_context.confirm)(
                    "Are you sure you want to quit without picking an Item Reward?".into(),
                    do_confirm_end.clone(),
                );
            }
        }
    };

    view! {
        <div class="max-w-4xl max-h-full">
            <div class="bg-zinc-800 rounded-md p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col gap-2 max-h-full">
                <div class="px-4 relative z-10 flex items-center justify-between">
                    <PanelTitle>"Grind Ended"</PanelTitle>
                    <CloseButton on:click=move |_| open.set(false) />
                </div>

                <div class="flex flex-col
                bg-neutral-900 rounded-lg shadow-[inset_0_0_24px_rgba(0,0,0,0.6)]
                p-2 xl:p-4 ring-1 ring-zinc-900">

                    <div class="flex">
                        <GoldCounter value=gold_donation_value />
                        "Collected"
                    </div>

                    <div class="flex">
                        <GemsCounter value=gems_value />
                        "Collected"
                    </div>

                    <div class="flex">
                        <ShardsCounter value=shards_value />
                        "Collected"
                    </div>

                    <div class="flex justify-between px-6 text-sm xl:text-base">
                        <span class="text-gray-400">"Total Time:"</span>
                        <span class="text-amber-100 font-medium font-number">
                            {move || format_duration(stats().elapsed_time, true)}
                        </span>
                    </div>
                    <div class="flex justify-between px-6 text-sm xl:text-base">
                        <span class="text-gray-400">"Area completed:"</span>
                        <span class="text-amber-100 font-medium font-number">{area_completed}</span>
                    </div>

                    <div>
                        <span>"Pick a Reward"</span>
                        <div class="flex">
                            {move || {
                                game_context
                                    .quest_rewards
                                    .get()
                                    .map(|quest_rewards| {
                                        quest_rewards
                                            .item_rewards
                                            .into_iter()
                                            .enumerate()
                                            .map(|(index, item_reward)| {
                                                view! {
                                                    <div
                                                        on:click=move |_| {
                                                            item_reward_picked
                                                                .update(|item_reward_picked| {
                                                                    *item_reward_picked = if *item_reward_picked == Some(index)
                                                                    {
                                                                        None
                                                                    } else {
                                                                        Some(index)
                                                                    };
                                                                });
                                                        }
                                                        class:brightness-30=move || {
                                                            item_reward_picked.get() != Some(index)
                                                        }
                                                    >
                                                        <ItemCard item_specs=Arc::new(item_reward) />
                                                    </div>
                                                }
                                            })
                                            .collect::<Vec<_>>()
                                    })
                            }}
                        </div>
                    </div>

                    <div>
                        <MenuButton on:click=try_confirm_end>"Confirm"</MenuButton>
                    </div>
                </div>
            </div>
        </div>
    }
}
