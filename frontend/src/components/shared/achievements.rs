use leptos::prelude::*;
use shared::data::{achievements::AchievementReward, cosmetics::CosmeticType, user::UserUnlocks};

use crate::{
    assets::img_asset,
    components::{
        data_context::DataContext,
        ui::{
            card::{CardHeader, CardInset, MenuCard},
            list_row::MenuListRow,
            menu_panel::MenuPanel,
            number::format_local_date,
            toast::{ToastVariant, Toasts, show_toast},
        },
    },
};

#[component]
pub fn AchievementsPanel(
    open: RwSignal<bool>,
    #[prop(into)] user_unlocks: Signal<UserUnlocks>,
) -> impl IntoView {
    let data_context = expect_context::<DataContext>();
    view! {
        <MenuPanel open w_full=false h_full=false class:items-center>
            <MenuCard class="w-full max-w-5xl mx-auto">
                <CardHeader title="Achievements" on_close=move || open.set(false) />
                <CardInset class="min-h-0 overflow-y-auto">
                    <div class="grid grid-cols-1 items-stretch gap-2 xl:grid-cols-2 xl:gap-3">
                        <For
                            each=move || {
                                data_context
                                    .achievements
                                    .read()
                                    .iter()
                                    .map(|(id, specs)| (id.clone(), specs.clone()))
                                    .collect::<Vec<_>>()
                            }
                            key=|(id, _)| id.clone()
                            children=move |(id, achievement)| {
                                let achieved_id = id.clone();
                                let achieved = Memo::new(move |_| {
                                    user_unlocks.read().achievements.contains_key(&achieved_id)
                                });
                                let unlocked_id = id.clone();
                                let unlocked_at = Memo::new(move |_| {
                                    user_unlocks.read().achievements.get(&unlocked_id).cloned()
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

                                view! {
                                    <MenuListRow
                                        class="h-full min-h-32 overflow-hidden"
                                        selected=Signal::from(achieved)
                                        class:brightness-50=move || !achieved.get()
                                    >
                                        <article class="flex min-h-32 gap-3 p-3 text-left">
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
                                                </div>
                                                <div class="mt-1 flex-1 text-sm leading-snug text-zinc-300">
                                                    {achievement.description}
                                                </div>
                                                <div class="mt-2 flex items-end justify-between gap-3 pt-2 text-xs">
                                                    <div class="min-w-0">
                                                        <span class="font-semibold tracking-wide text-zinc-500">
                                                            "Reward: "
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
                                                            })
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
