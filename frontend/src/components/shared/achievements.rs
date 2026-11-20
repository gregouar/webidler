use chrono::{DateTime, Utc};
use leptos::{prelude::*, task::spawn_local};
use shared::{
    data::{
        achievements::{AchievementCategory, AchievementReward, AchievementSpecs},
        cosmetics::CosmeticType,
        user::UserUnlocks,
    },
    http::server::ClaimRewardAchievementResponse,
};
use std::collections::HashMap;

use crate::{
    assets::img_asset,
    components::{
        backend_client::BackendClient,
        data_context::DataContext,
        ui::{
            buttons::{MenuButton, TabButton},
            card::{CardHeader, CardInset, MenuCard},
            checkbox::Checkbox,
            list_row::MenuListRow,
            menu_panel::MenuPanel,
            number::{comma_format, format_local_date},
            toast::{ToastVariant, Toasts, show_toast},
        },
    },
};

const ACHIEVEMENT_CATEGORIES: [AchievementCategory; 2] =
    [AchievementCategory::Global, AchievementCategory::Grind];

#[component]
pub fn AchievementsPanel(
    open: RwSignal<bool>,
    #[prop(into)] user_unlocks: RwSignal<UserUnlocks>,
    #[prop(optional, into)] achievements: Option<Signal<HashMap<String, DateTime<Utc>>>>,
    #[prop(optional, into)] achievements_progress: Option<Signal<HashMap<String, u64>>>,
) -> impl IntoView {
    let data_context = expect_context::<DataContext>();
    let active_category = RwSignal::new(AchievementCategory::Global);
    let display_user_achievements = RwSignal::new(false);
    let displayed_achievements = Memo::new(move |_| match achievements {
        Some(achievements) if !display_user_achievements.get() => achievements.get(),
        _ => user_unlocks.read().achievements.clone(),
    });

    let achieved_count = move || {
        data_context
            .achievements
            .read()
            .iter()
            .filter(|(achievement_id, specs)| {
                specs.category == active_category.get()
                    && displayed_achievements.read().contains_key(*achievement_id)
            })
            .count()
    };

    view! {
        <MenuPanel open w_full=false h_full=true class:items-center>
            <MenuCard class="h-full w-5xl mx-auto" gap=false>
                <CardHeader title="Achievements" on_close=move || open.set(false)>
                    <div class="flex self-end justify-center h-full ml-2 xl:ml-4 gap-2 xl:gap-4 overflow-clip">
                        {ACHIEVEMENT_CATEGORIES
                            .into_iter()
                            .map(|category| {
                                view! {
                                    <TabButton
                                        is_active=Signal::derive(move || {
                                            active_category.get() == category
                                        })
                                        on:click=move |_| active_category.set(category)
                                    >
                                        {achievement_category_label(category)}
                                    </TabButton>
                                }
                            })
                            .collect_view()}
                    </div>
                    <div class="flex-1" />
                    {achievements
                        .is_some()
                        .then(|| {
                            view! {
                                <Checkbox
                                    label="User achievements".to_string()
                                    checked=Signal::from(display_user_achievements)
                                    on_change=move |checked| {
                                        display_user_achievements.set(checked)
                                    }
                                />
                                <div class="flex-1" />
                            }
                        })}
                    <div class="w-20 flex justify-end text-shadow-md shadow-gray-950 text-zinc-400 text-xs xl:text-base font-medium">
                        {achieved_count}" / "
                        {move || {
                            data_context
                                .achievements
                                .read()
                                .values()
                                .filter(|specs| specs.category == active_category.get())
                                .count()
                        }}
                    </div>
                </CardHeader>
                <CardInset class="h-full">
                    <div class="grid grid-cols-1 items-stretch gap-2 xl:grid-cols-2 xl:gap-3">
                        <For
                            each=move || {
                                data_context
                                    .achievements
                                    .read()
                                    .iter()
                                    .filter(|(_, specs)| specs.category == active_category.get())
                                    .map(|(id, specs)| (id.clone(), specs.clone()))
                                    .collect::<Vec<_>>()
                            }
                            key=|(id, _)| id.clone()
                            children=move |(id, achievement)| {
                                let achieved_id = id.clone();
                                let achieved = Memo::new(move |_| {
                                    displayed_achievements.read().contains_key(&achieved_id)
                                });
                                let achieved_id = id.clone();
                                let character_achieved = Memo::new(move |_| {
                                    achievements
                                        .map(|achievements| {
                                            achievements.read().contains_key(&achieved_id)
                                        })
                                });
                                let unlocked_id = id.clone();
                                let unlocked_at = Memo::new(move |_| {
                                    displayed_achievements.read().get(&unlocked_id).cloned()
                                });
                                let required_goal_count = achievement.required_goal_count();
                                let progress_id = id.clone();
                                let goal_progress = Memo::new(move |_| {
                                    achievements_progress
                                        .zip(character_achieved.get())
                                        .map(|(achievements_progress, character_achieved)| {
                                            if character_achieved {
                                                required_goal_count
                                            } else {
                                                achievements_progress
                                                    .read()
                                                    .get(&progress_id)
                                                    .copied()
                                                    .unwrap_or_default()
                                                    .min(required_goal_count)
                                            }
                                        })
                                });
                                let rewards = {
                                    let achievement_rewards = achievement.rewards.clone();
                                    Memo::new(move |_| {
                                        achievement_rewards
                                            .iter()
                                            .map(|reward| reward_label(
                                                reward,
                                                achieved.get(),
                                                data_context,
                                            ))
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    })
                                };
                                let rewards_available = Memo::new({
                                    let achievement = achievement.clone();
                                    move |_| {
                                        achieved.get()
                                            && user_unlocks
                                                .with(|user_unlocks| {
                                                    is_claimable(&achievement, user_unlocks)
                                                })
                                    }
                                });

                                // (!display_user_achievements.get())
                                // .then_some(achievements_progress)
                                // .flatten()
                                // .map(|achievements_progress| {
                                // if achieved.get() {
                                // required_goal_count
                                // } else {
                                // achievements_progress
                                // .read()
                                // .get(&progress_id)
                                // .copied()
                                // .unwrap_or_default()
                                // .min(required_goal_count)
                                // }

                                view! {
                                    <MenuListRow
                                        class="h-full min-h-32 overflow-hidden"
                                        selected=Signal::from(achieved)
                                        class:brightness-50=move || !achieved.get()
                                    >
                                        <article class="flex h-full min-h-32 gap-3 p-3 text-left">
                                            <div
                                                class="flex h-14 w-14 shrink-0 items-center justify-center overflow-hidden"
                                                class:brightness-50=move || !achieved.get()
                                            >
                                                <img
                                                    src=img_asset(&achievement.icon)
                                                    class="h-full w-full object-cover"
                                                    style="filter: invert(1);"
                                                />
                                            </div>
                                            <div class="flex min-w-0 flex-1 flex-col">
                                                <div class="flex items-start gap-2">
                                                    <div class=move || {
                                                        format!(
                                                            "font-display text-base font-semibold xl:text-lg {}",
                                                            if achieved.get() {
                                                                "text-amber-200"
                                                            } else {
                                                                "text-zinc-500"
                                                            },
                                                        )
                                                    }>{achievement.name}</div>
                                                    {move || {
                                                        goal_progress
                                                            .get()
                                                            .map(|goal_progress| {
                                                                view! {
                                                                    <span class=move || {
                                                                        format!(
                                                                            "ml-auto shrink-0 text-sm font-semibold {}",
                                                                            if character_achieved.get().unwrap_or(achieved.get()) {
                                                                                "text-amber-200"
                                                                            } else {
                                                                                "text-zinc-300"
                                                                            },
                                                                        )
                                                                    }>
                                                                        {move || {
                                                                            format!(
                                                                                "{} / {}",
                                                                                comma_format(goal_progress as f64),
                                                                                comma_format(required_goal_count as f64),
                                                                            )
                                                                        }}
                                                                    </span>
                                                                }
                                                            })
                                                    }}

                                                </div>
                                                <div class="mt-1 flex-1 text-sm leading-snug text-zinc-300">
                                                    {achievement.description}
                                                </div>
                                                <div class="mt-2 flex min-h-[38px] items-end justify-between gap-3 pt-2 text-xs xl:min-h-[50px]">
                                                    <div class="min-w-0">
                                                        <span class="font-semibold tracking-wide text-zinc-500">
                                                            "Rewards: "
                                                        </span>
                                                        <span class=move || {
                                                            if achieved.get() {
                                                                "text-amber-300"
                                                            } else {
                                                                "text-zinc-400"
                                                            }
                                                        }>{move || rewards.get()}</span>
                                                    </div>
                                                    {move || {
                                                        let achievement_id = id.clone();
                                                        if rewards_available.get() {
                                                            Some(
                                                                view! {
                                                                    <MenuButton on:click=move |_| claim_rewards(
                                                                        user_unlocks,
                                                                        achievement_id.clone(),
                                                                    )>"Claim"</MenuButton>
                                                                }
                                                                    .into_any(),
                                                            )
                                                        } else {
                                                            unlocked_at
                                                                .get()
                                                                .map(|unlocked_at| {
                                                                    view! {
                                                                        <time
                                                                            class="shrink-0 text-right text-zinc-500"
                                                                            datetime=unlocked_at.to_rfc3339()
                                                                        >
                                                                            {format_local_date(unlocked_at)}
                                                                        </time>
                                                                    }
                                                                        .into_any()
                                                                })
                                                        }
                                                    }}
                                                </div>
                                            </div>
                                        </article>
                                    </MenuListRow>
                                }
                            }
                        />
                    </div>
                </CardInset>
            </MenuCard>
        </MenuPanel>
    }
}

fn achievement_category_label(category: AchievementCategory) -> &'static str {
    match category {
        AchievementCategory::Global => "Global",
        AchievementCategory::Grind => "Grind",
    }
}

pub fn notify_newly_unlocked_achievements(
    data_context: DataContext,
    toaster: Toasts,
    achievement_ids: Vec<String>,
) {
    for achievement_id in achievement_ids {
        let name = data_context
            .achievements
            .read_untracked()
            .get(&achievement_id)
            .map(|achievement| achievement.name.clone())
            .unwrap_or(achievement_id);
        show_toast(
            toaster,
            format!("Achievement unlocked: {name}!"),
            ToastVariant::Achievement,
        );
    }
}

fn reward_label(reward: &AchievementReward, achieved: bool, data: DataContext) -> String {
    match reward {
        AchievementReward::Cosmetic(id) => match data.cosmetics_specs.read().get(id) {
            Some(CosmeticType::Badge(specs)) => reward_value("Badge", &specs.name, achieved),
            Some(CosmeticType::Portrait(_)) => "Portrait".into(),
            Some(CosmeticType::Title(title)) => reward_value("Title", title, achieved),
            None => "Cosmetic".into(),
        },
        AchievementReward::Pet(id) => data
            .pets_specs
            .read()
            .get(id)
            .map(|pet| reward_value("Pet", &pet.name, achieved))
            .unwrap_or_else(|| "Pet".into()),
    }
}

fn reward_value(kind: &str, value: &str, achieved: bool) -> String {
    if achieved {
        format!("{kind} \"{value}\"")
    } else {
        kind.into()
    }
}

fn claim_rewards(user_unlocks: RwSignal<UserUnlocks>, achievement_id: String) {
    let backend = expect_context::<BackendClient>();
    let toaster = expect_context::<Toasts>();
    // let previous = player_pets.get_untracked();

    // let mut updated = previous.clone();
    // if let Some(pet) = pet {
    //     updated.retain(|_, assigned_pet| assigned_pet != &pet);
    //     updated.insert(button, pet);
    // } else {
    //     updated.remove(&button);
    // }

    // player_pets.set(updated.clone());
    spawn_local(async move {
        match backend.post_claim_reward_achievement(&achievement_id).await {
            Ok(ClaimRewardAchievementResponse {
                user_unlocks: new_user_unlocks,
            }) => user_unlocks.set(new_user_unlocks),
            Err(error) => {
                show_toast(
                    toaster,
                    format!("Failed to assign pet: {error}"),
                    ToastVariant::Error,
                );
            }
        }
    });
}

fn is_claimable(achievement: &AchievementSpecs, user_unlocks: &UserUnlocks) -> bool {
    !achievement.rewards.iter().all(|reward| match reward {
        AchievementReward::Cosmetic(cosmetic_id) => user_unlocks.cosmetics.contains(cosmetic_id),
        AchievementReward::Pet(pet_id) => user_unlocks.pets.contains(pet_id),
    })
}

pub fn are_rewards_available(user_unlock: RwSignal<UserUnlocks>) -> bool {
    let data_context: DataContext = expect_context();
    let achievements = data_context.achievements.read();
    let user_unlocks = user_unlock.read();
    for achievement_id in user_unlocks.achievements.keys() {
        if let Some(achievement) = achievements.get(achievement_id)
            && is_claimable(achievement, &user_unlocks)
        {
            return true;
        }
    }
    false
}
