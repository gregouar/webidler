use std::sync::Arc;

use indexmap::IndexSet;
use leptos::{html::*, prelude::*};

use crate::components::{
    data_context::DataContext,
    game::{GameContext, websocket::WebsocketContext},
    icons::battle_scene::RushIcon,
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
        number::{NumberInset, format_duration},
        tooltip::{StaticTooltip, StaticTooltipPosition},
    },
};
use shared::{
    computations,
    constants::{self, ITEM_REWARDS_MAP_MIN_LEVEL, ITEM_REWARDS_MIN_LEVEL},
    messages::client::{ClientMessage, TerminateGrindMessage},
};

#[component]
pub fn EndGrindPanel() -> impl IntoView {
    let game_context: GameContext = expect_context();

    let open = game_context.open_end_grind;

    Effect::new(move || {
        if game_context.grind_rewards.read().is_some() {
            open.set(true);
        }
    });

    view! {
        <MenuPanel open w_full=false h_full=false class:items-center>
            <EndGrind open />
        </MenuPanel>
    }
}

#[component]
fn EndGrind(open: RwSignal<bool>) -> impl IntoView {
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
    let stamina_kept =
        Signal::derive(move || computations::stamina_spill(game_context.player_stamina.get()));

    let area_completed = move || game_context.area_state.read().max_area_level;

    let item_rewards_picked = RwSignal::new(IndexSet::new());
    let end_quest_requested = RwSignal::new(false);
    let return_to_town_requested = RwSignal::new(false);

    let do_confirm_end = Arc::new({
        let conn: WebsocketContext = expect_context();
        move || {
            return_to_town_requested.set(true);
            conn.send(
                &TerminateGrindMessage {
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
                    .grind_rewards
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
            if game_context.grind_rewards.read_untracked().is_none() {
                end_quest_requested.set(true);
                conn.send(&ClientMessage::EndGrind);
            }

            if !rewards_expected(&game_context) {
                return_to_town_requested.set(true);
                conn.send(
                    &TerminateGrindMessage {
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
            if game_context.grind_rewards.read_untracked().is_some() {
                try_confirm_end();
            }
        }
    };

    Effect::new(move || {
        if open.get() && !return_to_town_requested.get_untracked() {
            item_rewards_picked.set(Default::default());
            if game_context.grind_rewards.read_untracked().is_none() {
                end_quest_requested.set(false);
            }
        }
    });

    view! {
        <MenuCard class="max-w-4xl max-h-full mx-auto">
            <CardHeader title="End Grind" on_close=move || open.set(false) />

            <CardInset>
                <div class="grid grid-cols-4 gap-4 text-center">
                    <div class="flex flex-col items-center gap-1">
                        <GoldCounter value=gold_donation_value w_full=true />
                    </div>
                    <div class="flex flex-col items-center gap-1">
                        <GemsCounter value=gems_value w_full=true />
                    </div>
                    <div class="flex flex-col items-center gap-1">
                        <ShardsCounter value=shards_value w_full=true />
                    </div>
                    <div class="flex items-center justify-center">
                        <StaticTooltip
                            position=StaticTooltipPosition::Bottom
                            tooltip=|| {
                                view! {
                                    {format!(
                                        "{:.0}% of Stamina kept",
                                        constants::STAMINA_SPILL_PERCENT * 100.0,
                                    )}
                                }
                            }
                        >
                            <div class=move || {
                                format!(
                                    "flex items-center gap-1 text-sm xl:text-xl font-number font-semibold text-amber-300 {}",
                                    if stamina_kept.get() < std::time::Duration::from_secs(60) {
                                        "grayscale"
                                    } else {
                                        ""
                                    },
                                )
                            }>
                                <NumberInset>
                                    <div class="text-right w-[8ch]">
                                        {move || format_duration(stamina_kept.get(), false)}
                                    </div>
                                </NumberInset>
                                <span class="text-base xl:text-2xl">
                                    <RushIcon />
                                </span>
                            </div>
                        </StaticTooltip>
                    </div>
                </div>

                <div class="h-px bg-gradient-to-r from-transparent via-zinc-700 to-transparent" />

                <div class="grid grid-cols-2 gap-x-8 gap-y-1 px-6 text-sm xl:text-base">
                    <div class="flex flex-col gap-1">
                        <div class="flex justify-between gap-4">
                            <span class="text-zinc-400">"Total Time"</span>
                            <span class="text-amber-100 font-medium font-number">
                                {move || format_duration(stats().elapsed_time, true)}
                            </span>
                        </div>
                        <div class="flex justify-between gap-4">
                            <span class="text-zinc-400">"Area Completed"</span>
                            <span class="text-amber-100 font-medium font-number">
                                {area_completed}
                            </span>
                        </div>
                        <Show when=move || {
                            !game_context.quest_completed.get()
                                && game_context.area_specs.read().quest.is_some()
                        }>
                            <div class="flex justify-between gap-4">
                                <span class="text-zinc-400">"Quest Level Goal"</span>
                                <span class="text-amber-100 font-medium font-number">
                                    {move || {
                                        game_context
                                            .area_specs
                                            .read()
                                            .quest
                                            .as_ref()
                                            .map(|quest| quest.area_level)
                                    }}
                                </span>
                            </div>
                        </Show>
                    </div>
                    <div class="flex flex-col gap-1">
                        <div class="flex justify-between gap-4">
                            <span class="text-zinc-400">"Monster Killed"</span>
                            <span class="text-amber-100 font-medium font-number">
                                {move || stats().monsters_killed}
                            </span>
                        </div>
                        <div class="flex justify-between gap-4">
                            <span class="text-zinc-400">"Player Deaths"</span>
                            <span class="text-amber-100 font-medium font-number">
                                {move || stats().player_deaths}
                            </span>
                        </div>
                        <Show when=move || {
                            !game_context.quest_completed.get()
                                && game_context.area_specs.read().quest.is_some()
                        }>
                            <div class="flex justify-between gap-4">
                                <span class="text-zinc-400">"Quest Completion"</span>
                                <span class=move || {
                                    let completed = game_context
                                        .area_specs
                                        .read()
                                        .quest
                                        .as_ref()
                                        .map(|quest| {
                                            game_context.area_state.read().max_area_level
                                                >= quest.area_level
                                        })
                                        .unwrap_or_default();
                                    if completed {
                                        "font-semibold text-emerald-400"
                                    } else {
                                        "font-semibold text-red-400"
                                    }
                                }>
                                    {move || {
                                        let completed = game_context
                                            .area_specs
                                            .read()
                                            .quest
                                            .as_ref()
                                            .map(|quest| {
                                                game_context.area_state.read().max_area_level
                                                    >= quest.area_level
                                            })
                                            .unwrap_or_default();
                                        if completed { "✓" } else { "✕" }
                                    }}
                                </span>
                            </div>
                        </Show>
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
                    if game_context.grind_rewards.read().is_none() {
                        view! {
                            <MenuButtonRed
                                on:click=primary_action
                                disabled=Signal::derive(move || {
                                    end_quest_requested.get() || return_to_town_requested.get()
                                })
                            >
                                {move || {
                                    if return_to_town_requested.get() {
                                        "Returning to Town..."
                                    } else if !rewards_expected(&game_context) {
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
                            <MenuButton
                                on:click=secondary_action
                                disabled=Signal::derive(move || { return_to_town_requested.get() })
                            >
                                {move || {
                                    if return_to_town_requested.get() {
                                        "Returning to Town..."
                                    } else {
                                        "Confirm Rewards & Return to Town"
                                    }
                                }}
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
                            .grind_rewards
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
                            .grind_rewards
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
                    <QuestItemReward />

                    <Show
                        when=move || {
                            game_context
                                .grind_rewards
                                .read()
                                .as_ref()
                                .map(|quest_rewards| !quest_rewards.item_rewards.is_empty())
                                .unwrap_or_default()
                        }
                        fallback=move || view! { <HiddenItemRewards /> }
                    >
                        {move || {
                            game_context
                                .grind_rewards
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
                                                            relative w-40 xl:w-48
                                                            transform-style-3d
                                                            reward-flip
                                                            "
                                                            style=move || {
                                                                let quest_offset = quest_reward_expected(&game_context)
                                                                    as usize;
                                                                format!(
                                                                    "animation-delay: {}ms",
                                                                    500 + (index + quest_offset) * 350,
                                                                )
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

                                                            <ItemRewardBackface rotate=true />
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

#[component]
fn QuestItemReward() -> impl IntoView {
    let game_context: GameContext = expect_context();

    view! {
        {move || {
            let quest_reached = !game_context.quest_completed.get()
                && game_context
                    .area_specs
                    .read()
                    .quest
                    .as_ref()
                    .map(|quest| game_context.area_state.read().max_area_level >= quest.area_level)
                    .unwrap_or_default();
            if !quest_reached {
                return None;
            }
            let revealed_reward = game_context
                .grind_rewards
                .read()
                .as_ref()
                .and_then(|rewards| rewards.quest_reward.clone());
            Some(
                if let Some(item_reward) = revealed_reward {

                    view! {
                        <div
                            class="perspective rounded-[8px]"
                            title="Quest reward — automatically collected"
                        >
                            <div class="relative w-40 xl:w-48 transform-style-3d reward-flip">
                                <div class="relative isolate overflow-clip rounded-[8px] border border-emerald-600/90 bg-zinc-900 shadow-[0_5px_14px_rgba(0,0,0,0.28),inset_0_0_0_1px_rgba(52,211,153,0.14)] backface-hidden">
                                    <ItemCard
                                        item_specs=Arc::new(item_reward)
                                        class:backface-hidden
                                    />
                                </div>
                                <ItemRewardBackface rotate=true green=true />
                            </div>
                        </div>
                    }
                        .into_any()
                } else {
                    view! {
                        <div
                            class="perspective rounded-[8px] opacity-95"
                            title="Quest reward — automatically collected"
                        >
                            <div class="relative w-40 xl:w-48 transform-style-3d">
                                <div class="invisible relative aspect-[2/3] rounded-[8px] border border-transparent"></div>
                                <ItemRewardBackface green=true />
                            </div>
                        </div>
                    }
                        .into_any()
                },
            )
        }}
    }
}

#[component]
fn ItemRewardBackface(
    #[prop(default = false)] rotate: bool,
    #[prop(default = false)] green: bool,
) -> impl IntoView {
    view! {
        <div class=move || {
            format!(
                "
                absolute inset-0
                backface-hidden
                isolate overflow-clip rounded-[8px]
                border {}
                bg-zinc-900
                shadow-[0_5px_14px_rgba(0,0,0,0.28),inset_0_1px_0_rgba(255,255,255,0.04),inset_0_-1px_0_rgba(0,0,0,0.35)]
                {}
                ",
                if green { "border-emerald-600/90" } else { "border-[#6c5329]/85" },
                if rotate { "rotate-y-180" } else { Default::default() },
            )
        }>
            <div class=move || {
                format!(
                    "
            absolute inset-0
            {}
            shadow-[0_5px_14px_rgba(0,0,0,0.28),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(230,208,154,0.18),inset_0_-1px_0_rgba(0,0,0,0.42)]
            flex items-center justify-center
            {} text-8xl font-display
            ",
                    if green {
                        "bg-[linear-gradient(180deg,rgba(52,211,153,0.12),rgba(0,0,0,0.18)),linear-gradient(180deg,rgba(35,48,44,0.96),rgba(18,24,22,1))]"
                    } else {
                        "bg-[linear-gradient(180deg,rgba(214,177,102,0.08),rgba(0,0,0,0.18)),linear-gradient(180deg,rgba(43,40,46,0.96),rgba(20,19,23,1))]"
                    },
                    if green { "text-emerald-400" } else { "text-amber-200" },
                )
            }>
                <span class="relative z-10 drop-shadow-[0_2px_0_rgba(0,0,0,0.55)]">"?"</span>
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

fn quest_reward_expected(game_context: &GameContext) -> bool {
    !game_context.quest_completed.get_untracked()
        && game_context
            .area_specs
            .read_untracked()
            .quest
            .as_ref()
            .map(|quest| {
                game_context.area_state.read_untracked().max_area_level >= quest.area_level
            })
            .unwrap_or_default()
}

fn rewards_expected(game_context: &GameContext) -> bool {
    let area_state = game_context.area_state.read();
    let area_specs = game_context.area_specs.read();
    predicted_item_rewards_amount(
        area_state.max_area_level,
        area_specs.training,
        area_specs.reward_slots,
    ) > 0
        || (!game_context.quest_completed.get()
            && area_specs
                .quest
                .as_ref()
                .map(|quest| area_state.max_area_level >= quest.area_level)
                .unwrap_or_default())
}

#[component]
fn HiddenItemRewards() -> impl IntoView {
    let game_context: GameContext = expect_context();

    view! {
        {move || {
            let area_state = game_context.area_state.read();
            let area_specs = game_context.area_specs.read();
            let amount = predicted_item_rewards_amount(
                area_state.max_area_level,
                area_specs.training,
                area_specs.reward_slots,
            );
            let quest_reward_unlocked = !game_context.quest_completed.get()
                && area_specs
                    .quest
                    .as_ref()
                    .map(|quest| area_state.max_area_level >= quest.area_level)
                    .unwrap_or_default();
            if amount == 0 {
                if quest_reward_unlocked {
                    ().into_any()
                } else {
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
                }
            } else {
                view! {
                    <div class="flex w-full flex-row gap-4 items-center justify-center">
                        {(0..amount)
                            .map(|_| {
                                view! {
                                    <div class="perspective rounded-[8px] opacity-90">
                                        <div class="
                                        relative w-40 xl:w-48
                                        transform-style-3d
                                        ">
                                            <div class="invisible relative rounded-[8px] border border-transparent">
                                                <div class="w-full aspect-[2/3]"></div>
                                            </div>
                                            <ItemRewardBackface />
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
