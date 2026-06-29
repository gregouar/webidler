use std::sync::Arc;

use indexmap::IndexSet;
use leptos::{html::*, prelude::*};

use crate::components::{
    data_context::DataContext,
    game::{GameContext, websocket::WebsocketContext},
    shared::{
        item_card::ItemCard,
        resources::{GemsCounter, GoldCounter, ShardsCounter},
        skills::SkillMasteryCard,
    },
    ui::{
        buttons::{MenuButton, MenuButtonRed},
        card::{CardHeader, CardInset, MenuCard},
        confirm::ConfirmContext,
        menu_panel::MenuPanel,
        number::format_duration,
    },
};
use shared::{
    computations,
    constants::{self, ITEM_REWARDS_MAP_MIN_LEVEL, ITEM_REWARDS_MIN_LEVEL},
    messages::client::{ClientMessage, TerminateQuestMessage},
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
                *game_context.area_specs.read().item_level_modifier
                    + *game_context.area_specs.read().power_level,
                constants::MONSTER_REWARD_INCREASE_FACTOR,
            )
    });

    let gems_value = Signal::derive(move || game_context.player_resources.read().gems);
    let shards_value = Signal::derive(move || game_context.player_resources.read().shards);

    let area_completed = move || game_context.area_state.read().max_area_level;

    let item_rewards_picked = RwSignal::new(IndexSet::new());
    let end_quest_requested = RwSignal::new(false);

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

    let try_confirm_end = Arc::new({
        let confirm_context: ConfirmContext = expect_context();
        move || {
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
    });

    let primary_action = {
        let conn: WebsocketContext = expect_context();
        move |_| {
            if game_context.quest_rewards.read_untracked().is_none() {
                end_quest_requested.set(true);
                conn.send(&ClientMessage::EndQuest);
            }

            if predicted_item_rewards_amount(
                game_context.area_state.read_untracked().max_area_level,
                game_context.area_specs.read_untracked().training,
                game_context.area_specs.read_untracked().reward_slots,
            ) == 0
            {
                conn.send(
                    &TerminateQuestMessage {
                        reward_picks: Default::default(),
                    }
                    .into(),
                );
            }
        }
    };

    let secondary_action = {
        let try_confirm_end = try_confirm_end.clone();
        move |_| {
            if game_context.quest_rewards.read_untracked().is_some() {
                try_confirm_end();
            }
        }
    };

    Effect::new(move || {
        if open.get() {
            item_rewards_picked.set(Default::default());
            if game_context.quest_rewards.read_untracked().is_none() {
                end_quest_requested.set(false);
            }
        }
    });

    view! {
        <MenuCard class="max-w-4xl max-h-full mx-auto">
            <CardHeader title="End Grind" on_close=move || open.set(false) />

            <CardInset>
                <div class="grid grid-cols-3 gap-4 text-center">
                    <div class="flex flex-col items-center gap-1">
                        <GoldCounter value=gold_donation_value w_full=true />
                    </div>
                    <div class="flex flex-col items-center gap-1">
                        <GemsCounter value=gems_value w_full=true />
                    </div>
                    <div class="flex flex-col items-center gap-1">
                        <ShardsCounter value=shards_value w_full=true />
                    </div>
                </div>

                <div class="h-px bg-gradient-to-r from-transparent via-zinc-700 to-transparent" />

                <div class="flex flex-col gap-1 px-6 text-sm xl:text-base">
                    <div class="flex justify-between">
                        <span class="text-zinc-400">"Total Time"</span>
                        <span class="text-amber-100 font-medium font-number">
                            {move || format_duration(stats().elapsed_time, true)}
                        </span>
                    </div>
                    <div class="flex justify-between">
                        <span class="text-zinc-400">"Area Completed"</span>
                        <span class="text-amber-100 font-medium font-number">{area_completed}</span>
                    </div>
                </div>

                <SkillMasteryRewards />

                <ItemRewards item_rewards_picked class:mt-2 />

                // <Show when=move || game_context.quest_rewards.read().is_none()>
                <div class="px-4 py-2 text-xs xl:text-sm text-zinc-400">
                    "Stopping this Grind will end your current run and reveal your Item Rewards. Your area and character progression will be reset. You will return to Town, keeping all Items, Gems, and Power Shards collected."
                </div>
            // </Show>
            </CardInset>

            <div class="flex justify-center">
                {move || {
                    let primary_action = primary_action.clone();
                    let secondary_action = secondary_action.clone();
                    if game_context.quest_rewards.read().is_none() {
                        view! {
                            <MenuButtonRed
                                on:click=primary_action
                                disabled=Signal::derive(move || { end_quest_requested.get() })
                            >
                                {move || {
                                    if predicted_item_rewards_amount(
                                        game_context.area_state.read().max_area_level,
                                        game_context.area_specs.read().training,
                                        game_context.area_specs.read().reward_slots,
                                    ) == 0
                                    {
                                        "Confirm End Grind & Return to Town"
                                    } else if end_quest_requested.get() {
                                        "Revealing Rewards..."
                                    } else {
                                        "Confirm End Grind & Reveal Rewards"
                                    }
                                }}
                            </MenuButtonRed>
                        }
                            .into_any()
                    } else {
                        view! {
                            <MenuButton on:click=secondary_action>
                                "Confirm Rewards & Return to Town"
                            </MenuButton>
                        }
                            .into_any()
                    }
                }}
            </div>
        </MenuCard>
    }
}

#[component]
fn SkillMasteryRewards() -> impl IntoView {
    let game_context: GameContext = expect_context();
    let data_context: DataContext = expect_context();

    let skill_mastery_rewards = Memo::new(move |_| {
        game_context
            .player_specs
            .read()
            .character_specs
            .skills_specs
            .iter()
            .filter_map(|skill_specs| {
                let experience_gained = game_context
                    .player_resources
                    .read()
                    .skill_masteries_experience
                    .get(&skill_specs.skill_id)
                    .copied()
                    .unwrap_or_default();
                if experience_gained <= 0.0 {
                    return None;
                }

                let max_level = data_context
                    .skill_mastery_specs
                    .read()
                    .get(&skill_specs.skill_id)
                    .map(|mastery_specs| mastery_specs.max_level)
                    .unwrap_or_default();

                let previous_mastery = game_context
                    .player_base_specs
                    .read()
                    .skill_masteries
                    .masteries
                    .get(&skill_specs.skill_id)
                    .cloned()
                    .unwrap_or_default();
                let mut current_mastery = previous_mastery.clone();
                current_mastery.experience += experience_gained;
                let gained_levels = current_mastery
                    .level(max_level)
                    .saturating_sub(previous_mastery.level(max_level));

                Some((
                    skill_specs.clone(),
                    current_mastery,
                    gained_levels,
                    experience_gained,
                ))
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div class="w-full flex flex-col gap-2">
            // <div class="w-full flex justify-between px-4">
            // <span class="text-center text-sm xl:text-base font-semibold text-amber-300 tracking-wide">
            // "Skill Masteries"
            // </span>
            // </div>
            <div class="grid grid-cols-1 gap-2 grid-cols-4">
                {skill_mastery_rewards
                    .get()
                    .into_iter()
                    .take(4)
                    .map(|(skill_specs, skill_mastery_state, gained_levels, delta_experience)| {
                        view! {
                            <SkillMasteryCard
                                skill_specs
                                skill_mastery_state
                                level_delta=gained_levels
                                experience_gained=delta_experience
                                compact=true
                            />
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </div>
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
                    {move || {
                        if game_context
                            .quest_rewards
                            .read()
                            .as_ref()
                            .map(|quest_rewards| !quest_rewards.item_rewards.is_empty())
                            .unwrap_or_default()
                        {
                            "Pick a Reward"
                        } else {
                            "Item Rewards"
                        }
                    }}
                </span>

                <span class="text-center text-sm xl:text-base text-zinc-400 ">
                    {move || {
                        game_context
                            .quest_rewards
                            .read()
                            .as_ref()
                            .map(|quest_rewards| {
                                (!quest_rewards.item_rewards.is_empty())
                                    .then(|| {
                                        format!(
                                            "({:0}/{:0})",
                                            item_rewards_picked.read().len(),
                                            game_context.area_specs.read_untracked().reward_picks,
                                        )
                                    })
                            })
                    }}
                </span>
            </div>

            <div class="relative isolate w-full overflow-clip rounded-[10px] border border-[#3b3428]
            bg-[linear-gradient(180deg,rgba(226,193,122,0.05),rgba(0,0,0,0.02)_28%,rgba(0,0,0,0.14)_100%),linear-gradient(135deg,rgba(40,39,45,0.98),rgba(18,18,22,1))]
            shadow-[0_6px_16px_rgba(0,0,0,0.22),inset_0_1px_0_rgba(255,255,255,0.04),inset_0_-1px_0_rgba(0,0,0,0.35)]">
                <div class="pointer-events-none absolute inset-[1px] rounded-[9px] border border-white/5"></div>
                <div class="pointer-events-none absolute inset-x-4 top-0 h-px bg-gradient-to-r from-transparent via-[#edd39a]/40 to-transparent"></div>
                <div class="relative z-10 flex w-full flex-row gap-4 items-center justify-center p-4">

                    <Show
                        when=move || {
                            game_context
                                .quest_rewards
                                .read()
                                .as_ref()
                                .map(|quest_rewards| quest_rewards.item_rewards.len() > 0)
                                .unwrap_or_default()
                        }
                        fallback=move || view! { <HiddenItemRewards /> }
                    >
                        {move || {
                            game_context
                                .quest_rewards
                                .get()
                                .map(|quest_rewards| {
                                    view! {
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
                                                        perspective rounded-[8px]
                                                        transition-all duration-150
                                                        cursor-pointer
                                                        {}
                                                        ",
                                                                if is_selected() {
                                                                    "brightness-110 -translate-y-[1px]"
                                                                } else {
                                                                    "opacity-90 hover:opacity-100"
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
                                                            <div class=move || {
                                                                format!(
                                                                    "relative isolate overflow-clip rounded-[8px]
                                                                border
                                                                bg-[linear-gradient(180deg,rgba(226,193,122,0.05),rgba(0,0,0,0.02)_28%,rgba(0,0,0,0.14)_100%),linear-gradient(135deg,rgba(40,39,45,0.98),rgba(18,18,22,1))]
                                                                shadow-[0_5px_14px_rgba(0,0,0,0.28),inset_0_1px_0_rgba(255,255,255,0.04),inset_0_-1px_0_rgba(0,0,0,0.35)]
                                                                backface-hidden
                                                                {}",
                                                                    if is_selected() {
                                                                        "border-[#b28a4f] shadow-[0_8px_18px_rgba(0,0,0,0.34),inset_0_1px_0_rgba(244,225,181,0.08),inset_0_0_0_1px_rgba(214,177,102,0.18)]"
                                                                    } else {
                                                                        "border-[#3b3428]"
                                                                    },
                                                                )
                                                            }>
                                                                <div class="pointer-events-none absolute inset-[1px] rounded-[7px] border border-white/5"></div>
                                                                <div class="pointer-events-none absolute inset-x-3 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/40 to-transparent"></div>
                                                                <ItemCard
                                                                    item_specs=Arc::new(item_reward.clone())
                                                                    class:backface-hidden
                                                                />
                                                            </div>

                                                            <div class="
                                                            absolute inset-0
                                                            backface-hidden
                                                            isolate overflow-clip rounded-[8px]
                                                            border border-[#6c5329]/85
                                                            bg-[linear-gradient(180deg,rgba(214,177,102,0.08),rgba(0,0,0,0.18)),linear-gradient(180deg,rgba(43,40,46,0.96),rgba(20,19,23,1))]
                                                            shadow-[0_5px_14px_rgba(0,0,0,0.28),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(230,208,154,0.18),inset_0_-1px_0_rgba(0,0,0,0.42)]
                                                            flex items-center justify-center
                                                            text-amber-200 text-8xl font-display
                                                            rotate-y-180
                                                            ">
                                                                <div class="pointer-events-none absolute inset-[1px] rounded-[7px] border border-[#d5b16d]/18"></div>
                                                                <div class="pointer-events-none absolute inset-x-4 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/45 to-transparent"></div>
                                                                <div class="pointer-events-none absolute inset-[6px] rounded-[6px] border border-black/45 bg-[linear-gradient(180deg,rgba(10,10,12,0.78),rgba(28,26,32,0.92))] shadow-[inset_0_2px_5px_rgba(0,0,0,0.5),inset_0_1px_0_rgba(255,255,255,0.03)]"></div>
                                                                <span class="relative z-10 drop-shadow-[0_2px_0_rgba(0,0,0,0.55)]">
                                                                    "?"
                                                                </span>
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            })
                                            .collect::<Vec<_>>()}
                                    }
                                })
                        }}
                    </Show>
                </div>
            </div>
        </div>
    }
}

fn predicted_item_rewards_amount(area_level: u16, training: bool, reward_slots: u8) -> u8 {
    if area_level >= ITEM_REWARDS_MIN_LEVEL && !training {
        reward_slots
    } else {
        0
    }
}

#[component]
fn HiddenItemRewards() -> impl IntoView {
    let game_context: GameContext = expect_context();

    view! {
        {move || {
            let amount = predicted_item_rewards_amount(
                game_context.area_state.read().max_area_level,
                game_context.area_specs.read().training,
                game_context.area_specs.read().reward_slots,
            );
            if amount == 0 {
                view! {
                    <div class="flex-1 text-zinc-400">
                        {format!(
                            "Complete at least {} Areas to get an Item Reward, and at least {} to get a guaranteed Edict Item drop.",
                            ITEM_REWARDS_MIN_LEVEL,
                            ITEM_REWARDS_MAP_MIN_LEVEL,
                        )}
                    </div>
                }
                    .into_any()
            } else {
                view! {
                    <div class="flex w-full flex-row gap-4 items-center justify-center">
                        {(0..amount)
                            .map(|_| {
                                view! {
                                    <div class="perspective rounded-[8px] opacity-90">
                                        <div class="
                                        relative w-full h-full max-w-48
                                        transform-style-3d
                                        ">
                                            <div class="
                                            relative min-h-64 min-w-40
                                            backface-hidden
                                            isolate overflow-clip rounded-[8px]
                                            border border-[#6c5329]/85
                                            bg-[linear-gradient(180deg,rgba(214,177,102,0.08),rgba(0,0,0,0.18)),linear-gradient(180deg,rgba(43,40,46,0.96),rgba(20,19,23,1))]
                                            shadow-[0_5px_14px_rgba(0,0,0,0.28),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(230,208,154,0.18),inset_0_-1px_0_rgba(0,0,0,0.42)]
                                            flex items-center justify-center
                                            text-amber-200 text-8xl font-display
                                            ">
                                                <div class="pointer-events-none absolute inset-[1px] rounded-[7px] border border-[#d5b16d]/18"></div>
                                                <div class="pointer-events-none absolute inset-x-4 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/45 to-transparent"></div>
                                                <div class="pointer-events-none absolute inset-[6px] rounded-[6px] border border-black/45 bg-[linear-gradient(180deg,rgba(10,10,12,0.78),rgba(28,26,32,0.92))] shadow-[inset_0_2px_5px_rgba(0,0,0,0.5),inset_0_1px_0_rgba(255,255,255,0.03)]"></div>
                                                <span class="relative z-10 drop-shadow-[0_2px_0_rgba(0,0,0,0.55)]">
                                                    "?"
                                                </span>
                                            </div>
                                        </div>
                                    </div>
                                }
                            })
                            .collect::<Vec<_>>()}
                    </div>
                }
                    .into_any()
            }
        }}
    }
}
