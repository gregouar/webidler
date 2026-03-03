use std::sync::Arc;

use indexmap::IndexSet;
use leptos::{html::*, prelude::*};

use crate::components::{
    game::{GameContext, websocket::WebsocketContext},
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
};
use shared::{
    computations,
    constants::{self, ITEM_REWARDS_MAP_MIN_LEVEL, ITEM_REWARDS_MIN_LEVEL},
    messages::client::TerminateQuestMessage,
};

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
                game_context.area_specs.read().item_level_modifier
                    + game_context.area_specs.read().power_level,
                constants::MONSTER_REWARD_INCREASE_FACTOR,
            )
    });

    let gems_value = Signal::derive(move || game_context.player_resources.read().gems);
    let shards_value = Signal::derive(move || game_context.player_resources.read().shards);

    let area_completed = move || game_context.area_state.read().max_area_level;

    let item_rewards_picked = RwSignal::new(IndexSet::new());

    let do_confirm_end = Arc::new({
        let conn: WebsocketContext = expect_context();
        move || {
            conn.send(
                &TerminateQuestMessage {
                    reward_picks: item_rewards_picked
                        .get_untracked()
                        .into_iter()
                        .map(|x| x as u8)
                        .collect(),
                }
                .into(),
            );
        }
    });

    let try_confirm_end = {
        let confirm_context: ConfirmContext = expect_context();
        move |_| {
            if item_rewards_picked.read_untracked().len()
                == game_context.area_specs.read_untracked().reward_picks as usize
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
                    "Are you sure you want to quit without picking all your Item Rewards?".into(),
                    do_confirm_end.clone(),
                );
            }
        }
    };

    Effect::new(move || {
        if open.get() {
            item_rewards_picked.set(Default::default());
        }
    });

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

                <ItemRewards item_rewards_picked class:mt-2 />
            </CardInset>

            <div class="flex justify-center">
                <MenuButton on:click=try_confirm_end>"Confirm Reward & Exit"</MenuButton>
            </div>
        </Card>
    }
}

#[component]
fn ItemRewards(item_rewards_picked: RwSignal<IndexSet<usize>>) -> impl IntoView {
    let game_context: GameContext = expect_context();

    let pick_reward = move |index| {
        item_rewards_picked.update(|picked| {
            if picked.contains(&index) {
                picked.shift_remove(&index);
            } else {
                picked.insert(index);
                if picked.len() > game_context.area_specs.read_untracked().reward_picks as usize {
                    picked.shift_remove_index(0);
                }
            }
        });
    };

    // TODO: Make responsive on mobile

    view! {
        <div class="w-full h-full flex flex-col gap-2 items-center justify-center">

            <div class="w-full flex justify-between px-4">
                <span class="text-center text-sm xl:text-base font-semibold text-amber-300 tracking-wide">
                    "Pick a Reward"
                </span>

                <span class="text-center text-sm xl:text-base text-gray-400 ">
                    {move || {
                        (game_context
                            .quest_rewards
                            .with(|quest_rewards| {
                                quest_rewards
                                    .as_ref()
                                    .map(|quest_rewards| !quest_rewards.item_rewards.is_empty())
                                    .unwrap_or_default()
                            }))
                            .then(|| {
                                format!(
                                    "({:0}/{:0})",
                                    item_rewards_picked.read().len(),
                                    game_context.area_specs.read_untracked().reward_picks,
                                )
                            })
                    }}
                </span>
            </div>

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
                                                {format!(
                                                    "Complete at least {} Areas to get an Item Reward, and at least {} to get a guaranteed Edict Item drop.",
                                                    ITEM_REWARDS_MIN_LEVEL,
                                                    ITEM_REWARDS_MAP_MIN_LEVEL,
                                                )}
                                            </div>
                                        }
                                    })}
                                {quest_rewards
                                    .item_rewards
                                    .into_iter()
                                    .enumerate()
                                    .map(|(index, item_reward)| {
                                        let is_selected = move || {
                                            item_rewards_picked.read().contains(&index)
                                        };
                                        view! {
                                            <div
                                                class=move || {
                                                    format!(
                                                        "
                                                        perspective rounded-md
                                                        transition-all duration-150
                                                        cursor-pointer
                                                        {}
                                                        ",
                                                        if is_selected() {
                                                            "shadow-[0_0_8px_gold] brightness-110"
                                                        } else {
                                                            "opacity-80"
                                                        },
                                                    )
                                                }
                                                on:click=move |_| pick_reward(index)
                                            >
                                                <div
                                                    class="
                                                    relative w-full h-full max-w-48
                                                    transform-style-3d
                                                    reward-flip
                                                    "
                                                    style=move || {
                                                        format!("animation-delay: {}ms", 500 + index * 350)
                                                    }
                                                >
                                                    <ItemCard
                                                        item_specs=Arc::new(item_reward.clone())
                                                        class:backface-hidden
                                                    />

                                                    <div class="
                                                    absolute inset-0
                                                    backface-hidden
                                                    rounded-md border-2 border-zinc-700 bg-gradient-to-br from-zinc-800 to-zinc-900
                                                    ring-stone-500
                                                    ring-1 xl:ring-2
                                                    rounded-md
                                                    flex items-center justify-center
                                                    text-amber-200 text-8xl
                                                    rotate-y-180
                                                    ">"?"</div>
                                                </div>
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
