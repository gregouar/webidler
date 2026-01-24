use std::sync::Arc;

use leptos::{html::*, prelude::*};

use crate::components::{
    game::GameContext,
    shared::{
        item_card::ItemCard,
        resources::{GemsCounter, GoldCounter, ShardsCounter},
    },
    ui::{
        buttons::MenuButton,
        card::{Card, CardHeader, CardInset},
        confirm::ConfirmContext,
        menu_panel::MenuPanel,
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
        <MenuPanel open w_full=false h_full=false class:items-center>
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
            if item_reward_picked.read_untracked().is_some()
                || game_context
                    .quest_rewards
                    .read_untracked()
                    .as_ref()
                    .map(|quest_rewards| quest_rewards.item_rewards.is_empty())
                    .unwrap_or_default()
            {
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
        <Card class="max-w-2xl max-h-full mx-auto">
            <CardHeader title="Grind Ended" on_close=move || open.set(false) />

            <CardInset>
                <div class="grid grid-cols-3 gap-4 text-center">
                    <div class="flex flex-col items-center gap-1">
                        <GoldCounter value=gold_donation_value />
                    </div>
                    <div class="flex flex-col items-center gap-1">
                        <GemsCounter value=gems_value />
                    </div>
                    <div class="flex flex-col items-center gap-1">
                        <ShardsCounter value=shards_value />
                    </div>
                </div>

                <div class="h-px bg-gradient-to-r from-transparent via-zinc-700 to-transparent" />

                <div class="flex flex-col gap-1 px-6 text-sm xl:text-base">
                    <div class="flex justify-between">
                        <span class="text-gray-400">"Total Time"</span>
                        <span class="text-amber-100 font-medium font-number">
                            {move || format_duration(stats().elapsed_time, true)}
                        </span>
                    </div>
                    <div class="flex justify-between">
                        <span class="text-gray-400">"Area Completed"</span>
                        <span class="text-amber-100 font-medium font-number">{area_completed}</span>
                    </div>
                </div>

                <ItemRewards item_reward_picked />
            </CardInset>

            <div class="flex justify-center">
                <MenuButton on:click=try_confirm_end>"Confirm Reward & Exit"</MenuButton>
            </div>
        </Card>
    }
}

#[component]
fn ItemRewards(item_reward_picked: RwSignal<Option<usize>>) -> impl IntoView {
    let game_context: GameContext = expect_context();

    // TODO: Make responsive on mobile

    view! {
        <div class="w-full h-full flex flex-col gap-2 items-center justify-center">

            <span class="text-center text-sm xl:text-base font-semibold text-amber-300 tracking-wide">
                "Pick a Reward"
            </span>

            <div class="w-full flex flex-row gap-4 items-center justify-center
            bg-neutral-800 rounded-lg  ring-1 ring-zinc-950  p-4">

                {move || {
                    game_context
                        .quest_rewards
                        .get()
                        .map(|quest_rewards| {
                            view! {
                                {quest_rewards
                                    .item_rewards
                                    .is_empty()
                                    .then(|| {
                                        view! {
                                            <div class="flex-1 text-gray-400">
                                                "Complete more Areas to get an Item Reward"
                                            </div>
                                        }
                                    })}
                                {quest_rewards
                                    .item_rewards
                                    .into_iter()
                                    .enumerate()
                                    .map(|(index, item_reward)| {
                                        view! {
                                            <div
                                                class="
                                                transition-all duration-150
                                                cursor-pointer
                                                "
                                                class:opacity-40=move || {
                                                    item_reward_picked.get() != Some(index)
                                                }
                                                class:ring-2=move || item_reward_picked.get() == Some(index)
                                                class:ring-amber-400=move || {
                                                    item_reward_picked.get() == Some(index)
                                                }
                                                class:rounded-md=true
                                                on:click=move |_| {
                                                    item_reward_picked
                                                        .update(|picked| {
                                                            *picked = if *picked == Some(index) {
                                                                None
                                                            } else {
                                                                Some(index)
                                                            };
                                                        });
                                                }
                                            >
                                                <ItemCard item_specs=Arc::new(item_reward) />
                                            </div>
                                        }
                                    })
                                    .collect::<Vec<_>>()}
                            }
                        })
                }}
            </div>
        </div>
    }
}
