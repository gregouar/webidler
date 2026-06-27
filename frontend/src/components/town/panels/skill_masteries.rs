use std::sync::Arc;

use leptos::{html::*, prelude::*, task::spawn_local};

use shared::{
    data::{
        skill::SkillType,
        skill_mastery::{
            PlayerSkillMasteries, SkillMasteryState, SkillMasteryUpgrade,
            SkillMasteryUpgradeEffect, SkillMasteryUpgradeEffectType,
        },
    },
    http::client::SaveSkillMasteriesRequest,
};
use strum::IntoEnumIterator;

use crate::components::{
    backend_client::BackendClient,
    data_context::DataContext,
    events::{EventsContext, Key},
    shared::{
        skills::{SkillMasteryCard, skill_specs_with_mastery},
        tooltips::{effects_tooltip, skill_tooltip},
    },
    town::TownContext,
    ui::{
        buttons::MenuButton,
        card::{CardHeader, CardInset, CardInsetTitle, MenuCard},
        list_row::MenuListRow,
        menu_panel::MenuPanel,
        toast::*,
    },
};

#[component]
pub fn SkillMasteriesPanel(
    open: RwSignal<bool>,
    #[prop(default = false)] view_only: bool,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let skill_masteries = RwSignal::new(PlayerSkillMasteries::default());

    let reset = move || {
        skill_masteries.set(town_context.player_skill_masteries.get_untracked());
    };

    Effect::new(move || {
        if open.get() {
            reset();
        }
    });

    let dirty = Signal::derive(move || {
        *town_context.player_skill_masteries.read() != *skill_masteries.read()
    });

    view! {
        <MenuPanel open=open w_full=false h_full=true class:items-center>
            <MenuCard class="max-w-6xl mx-auto h-full">
                <CardHeader title="Skill Masteries" on_close=move || open.set(false)>
                    {(!view_only)
                        .then(|| {
                            view! {
                                <div class="flex-1" />
                                <div class="flex h-full items-center gap-2">
                                    <MenuButton
                                        on:click=move |_| reset()
                                        disabled=Signal::derive(move || !dirty.get())
                                    >
                                        "Cancel"
                                    </MenuButton>
                                    <ConfirmButton skill_masteries open dirty />
                                </div>
                            }
                        })}
                </CardHeader>
                <CardInset>
                    <FavoriteSkillsPicker skill_masteries view_only />
                    <MasterySkillShop skill_masteries view_only />
                </CardInset>
            </MenuCard>
        </MenuPanel>
    }
}

#[component]
pub fn SkillMasteryDetailsModal(#[prop(default = false)] view_only: bool) -> impl IntoView {
    let town_context = expect_context::<TownContext>();
    let open = town_context.open_skill_mastery_details;
    let selected_skill = town_context.selected_skill_mastery;

    let skill_masteries = RwSignal::new(PlayerSkillMasteries::default());

    let reset = move || {
        skill_masteries.set(town_context.player_skill_masteries.get_untracked());
    };

    Effect::new(move || {
        if open.get() {
            reset();
        }
    });

    let dirty = Signal::derive(move || {
        *town_context.player_skill_masteries.read() != *skill_masteries.read()
    });

    view! {
        <MenuPanel open=open w_full=true h_full=true>
            <MenuCard class="w-full h-full">
                <CardHeader title="Skill Mastery" on_close=move || open.set(false)>
                    {(!view_only)
                        .then(|| {
                            view! {
                                <div class="flex-1" />
                                <div class="flex h-full items-center gap-2">
                                    <MenuButton
                                        on:click=move |_| {
                                            reset();
                                            open.set(false);
                                        }
                                        disabled=Signal::derive(move || !dirty.get())
                                    >
                                        "Cancel"
                                    </MenuButton>
                                    <ConfirmButton skill_masteries open dirty />
                                </div>
                            }
                        })}
                </CardHeader>
                <CardInset>
                    <MasteryUpgradePanel skill_masteries selected_skill view_only />
                </CardInset>
            </MenuCard>
        </MenuPanel>
    }
}

#[component]
fn ConfirmButton(
    skill_masteries: RwSignal<PlayerSkillMasteries>,
    open: RwSignal<bool>,
    dirty: Signal<bool>,
) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let toaster = expect_context::<Toasts>();
    let character_id = town_context.character.read_untracked().character_id;

    let save = move |_| {
        spawn_local(async move {
            match backend
                .post_save_skill_masteries(&SaveSkillMasteriesRequest {
                    character_id,
                    skill_masteries: skill_masteries.get_untracked(),
                })
                .await
            {
                Ok(response) => {
                    town_context
                        .player_skill_masteries
                        .set(response.skill_masteries);
                    town_context
                        .skill_mastery_skill_specs
                        .set(response.skill_mastery_skill_specs);
                    open.set(false);
                }
                Err(e) => show_toast(
                    toaster,
                    format!("Failed to save skill masteries: {e}"),
                    ToastVariant::Error,
                ),
            }
        });
    };

    view! {
        <MenuButton on:click=save disabled=Signal::derive(move || !dirty.get())>
            "Confirm"
        </MenuButton>
    }
}

#[component]
fn FavoriteSkillsPicker(
    skill_masteries: RwSignal<PlayerSkillMasteries>,
    view_only: bool,
) -> impl IntoView {
    let data_context = expect_context::<DataContext>();
    let town_context = expect_context::<TownContext>();

    let favorite_skill_slots = Memo::new(move |_| {
        let skill_specs = data_context.skill_specs.get();
        let skill_mastery_skill_specs = town_context.skill_mastery_skill_specs.get();
        let skill_masteries = skill_masteries.get();

        (0..4)
            .map(|index| {
                skill_masteries
                    .favorite_skills
                    .get(index)
                    .and_then(|skill_id| {
                        let mastery = skill_masteries.masteries.get(skill_id).cloned()?;
                        let base_skill_specs = skill_specs.get(skill_id)?;
                        let skill_specs = skill_specs_with_mastery(
                            skill_id.clone(),
                            base_skill_specs,
                            &skill_mastery_skill_specs,
                        );
                        Some((skill_id.clone(), mastery, skill_specs))
                    })
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div class="mb-4 space-y-2">
            <div class="flex items-center justify-center gap-3 px-1">
                <div class="h-[2px] flex-1 rounded-full bg-gradient-to-r from-transparent via-amber-300/70 to-transparent"></div>
                <h3 class="font-display text-sm xl:text-base tracking-[0.14em] uppercase text-amber-200">
                    "Favorites"
                </h3>
                <div class="h-[2px] flex-1 rounded-full bg-gradient-to-r from-transparent via-amber-300/70 to-transparent"></div>
            </div>
            <div class="grid grid-cols-2 md:grid-cols-4 gap-2 xl:gap-3">
                <For
                    each=move || 0..4
                    key=|index| *index
                    let:(index)
                >
                    {move || favorite_skill_slots.read().get(index).cloned().flatten()
                        .map(|(skill_id, skill_mastery_state, skill_specs)| {
                            let skill_id_for_click = skill_id.clone();
                            view! {
                                <div class="flex min-w-0 flex-col gap-2">
                                    <SkillMasteryCard
                                        skill_specs
                                        skill_mastery_state
                                        on_click=Callback::new(move |_| {
                                            town_context
                                                .selected_skill_mastery
                                                .set(Some(skill_id_for_click.clone()));
                                            town_context.open_skill_mastery_details.set(true);
                                        })
                                    />
                                    <FavoriteSkillButton skill_id skill_masteries view_only />
                                </div>
                            }
                                .into_any()
                        })
                        .unwrap_or_else(|| {
                            view! {
                                <div class="flex min-w-0 flex-col gap-2">
                                    <SkillMasteryCard empty_label=format!("Favorite {}", index + 1) />
                                    {(!view_only)
                                        .then(|| {
                                            view! {
                                                <MenuButton
                                                    disabled=Signal::derive(|| true)
                                                    class="w-full"
                                                >
                                                    "Remove Favorite"
                                                </MenuButton>
                                            }
                                        })}
                                </div>
                            }
                                .into_any()
                        })}
                </For>
            </div>
        </div>
    }
}

#[component]
fn MasterySkillShop(
    skill_masteries: RwSignal<PlayerSkillMasteries>,
    view_only: bool,
) -> impl IntoView {
    let data_context = expect_context::<DataContext>();
    let town_context = expect_context::<TownContext>();

    let available_skills = Memo::new(move |_| {
        let skill_specs = data_context.skill_specs.get();
        let skill_mastery_skill_specs = town_context.skill_mastery_skill_specs.get();
        let favorite_skills = skill_masteries.get().favorite_skills;
        let mut sections = skill_masteries
            .get()
            .masteries
            .into_iter()
            .filter_map(|(skill_id, mastery)| {
                if mastery.experience <= 0.0 || favorite_skills.contains(&skill_id) {
                    return None;
                }

                let base_skill_specs = skill_specs.get(&skill_id)?;
                let skill_specs = skill_specs_with_mastery(
                    skill_id.clone(),
                    base_skill_specs,
                    &skill_mastery_skill_specs,
                );
                Some((skill_id, mastery, skill_specs))
            })
            .fold(
                std::collections::HashMap::<SkillType, Vec<_>>::new(),
                |mut acc, skill| {
                    acc.entry(skill.2.skill_type).or_default().push(skill);
                    acc
                },
            );

        for section in sections.values_mut() {
            section.sort_by_key(|(_, _, skill_specs)| skill_specs.name.clone());
        }

        sections
    });
    let skill_sections = Memo::new(move |_| {
        available_skills.with(|available_skills| {
            SkillType::iter()
                .filter_map(|skill_type| {
                    let skills = available_skills
                        .get(&skill_type)
                        .cloned()
                        .unwrap_or_default();
                    (!skills.is_empty()).then_some((skill_type, skills))
                })
                .collect::<Vec<_>>()
        })
    });

    view! {
        <div class="space-y-3 xl:space-y-4">

            {move || {
                skill_sections
                    .get()
                    .into_iter()
                    .map(move |(skill_type, skills)| {
                        view! {
                            <div class="space-y-3 xl:space-y-4">
                                    <div class="flex items-center gap-3 px-1">
                                        <div class=format!(
                                            "h-[2px] flex-1 rounded-full bg-gradient-to-r from-transparent {} to-transparent",
                                            skill_type_glow(skill_type),
                                        )></div>
                                        <h3 class=format!(
                                            "font-display text-sm xl:text-base tracking-[0.14em] uppercase {}",
                                            skill_type_title_color(skill_type),
                                        )>
                                            {skill_type_title(skill_type)}
                                        </h3>
                                        <div class=format!(
                                            "h-[2px] flex-1 rounded-full bg-gradient-to-r from-transparent {} to-transparent",
                                            skill_type_glow(skill_type),
                                        )></div>
                                    </div>

                                    <div class="grid grid-cols-2 md:grid-cols-3 xl:grid-cols-4 gap-2 xl:gap-3">
                                        <For
                                            each=move || skills.clone().into_iter()
                                            key=|(skill_id, _, _)| skill_id.clone()
                                            let:((skill_id, skill_mastery_state, skill_specs))
                                        >
                                            {let skill_id_for_click = skill_id.clone();
                                            view! {
                                                <div class="flex min-w-0 flex-col gap-2">
                                                    <SkillMasteryCard
                                                        skill_specs
                                                        skill_mastery_state
                                                        on_click=Callback::new(move |_| {
                                                            town_context
                                                                .selected_skill_mastery
                                                                .set(Some(skill_id_for_click.clone()));
                                                            town_context.open_skill_mastery_details.set(true);
                                                        })
                                                    />
                                                    <FavoriteSkillButton skill_id skill_masteries view_only />
                                                </div>
                                            }}
                                        </For>
                                    </div>
                                </div>
                        }
                    })
                    .collect::<Vec<_>>()
            }}
        </div>
    }
}

#[component]
fn FavoriteSkillButton(
    skill_id: String,
    skill_masteries: RwSignal<PlayerSkillMasteries>,
    view_only: bool,
) -> impl IntoView {
    let is_favorite = Signal::derive({
        let skill_id = skill_id.clone();
        move || {
            skill_masteries
                .read()
                .favorite_skills
                .iter()
                .any(|favorite| favorite == &skill_id)
        }
    });
    let can_mark_favorite = Signal::derive(move || {
        !view_only && (is_favorite.get() || skill_masteries.read().favorite_skills.len() < 4)
    });
    let skill_id_for_toggle = skill_id.clone();

    view! {
        {(!view_only)
            .then(|| {
                view! {
                    <MenuButton
                        on:click=move |ev| {
                            ev.stop_propagation();
                            skill_masteries
                                .update(|skill_masteries| {
                                    if let Some(index) = skill_masteries
                                        .favorite_skills
                                        .iter()
                                        .position(|favorite| favorite == &skill_id_for_toggle)
                                    {
                                        skill_masteries.favorite_skills.remove(index);
                                    } else if skill_masteries.favorite_skills.len() < 4 {
                                        skill_masteries
                                            .favorite_skills
                                            .push(skill_id_for_toggle.clone());
                                    }
                                });
                        }
                        disabled=Signal::derive(move || !can_mark_favorite.get())
                        class="w-full"
                    >
                        {move || {
                            if is_favorite.get() { "Remove Favorite" } else { "Mark Favorite" }
                        }}
                    </MenuButton>
                }
            })}
    }
}

#[component]
fn MasteryUpgradePanel(
    skill_masteries: RwSignal<PlayerSkillMasteries>,
    selected_skill: RwSignal<Option<String>>,
    view_only: bool,
) -> impl IntoView {
    let data_context = expect_context::<DataContext>();
    let town_context = expect_context::<TownContext>();

    view! {
        <div class="min-w-0 min-h-0 overflow-y-auto">
            {move || {
                let Some(skill_id) = selected_skill.get() else {
                    return view! {
                        <div class="flex h-full items-center justify-center text-sm text-zinc-500">
                            "Select a skill."
                        </div>
                    }
                        .into_any();
                };
                let Some(mastery_specs) = data_context
                    .skill_mastery_specs
                    .read()
                    .get(&skill_id)
                    .cloned() else {
                    return view! {
                        <div class="flex h-full items-center justify-center text-sm text-zinc-500">
                            "No mastery upgrades available."
                        </div>
                    }
                        .into_any();
                };
                let skill_name = data_context.skill_name(&skill_id);
                let skill_specs = data_context
                    .skill_specs
                    .read()
                    .get(&skill_id)
                    .map(|base_skill_specs| {
                        skill_specs_with_mastery(
                            skill_id.clone(),
                            base_skill_specs,
                            &town_context.skill_mastery_skill_specs.read(),
                        )
                    });
                let skill_mastery_state = skill_masteries
                    .read()
                    .masteries
                    .get(&skill_id)
                    .cloned()
                    .unwrap_or_default();
                let mastery_level = skill_mastery_state.level().min(mastery_specs.max_level);
                let spent_points = spent_mastery_points(
                    &skill_mastery_state,
                    &mastery_specs.upgrades,
                );
                let upgrades = mastery_specs.upgrades.clone();
                let skill_id_for_reset = skill_id.clone();
                view! {
                    <section class="w-full min-w-0 space-y-4">
                        <div class="mx-auto w-full max-w-xs">
                            {skill_specs
                                .map(|skill_specs| {
                                    view! {
                                        <SkillMasteryCard
                                            skill_specs
                                            skill_mastery_state
                                        />
                                    }
                                        .into_any()
                                })
                                .unwrap_or_else(|| {
                                    view! {
                                        <SkillMasteryCard
                                            empty_label=skill_name.clone()
                                        />
                                    }
                                        .into_any()
                                })}
                        </div>

                        <div class="h-px bg-gradient-to-r from-transparent via-zinc-700 to-transparent" />

                        <div class="mb-2 mt-1 grid grid-cols-1 gap-2 px-1 text-xs xl:grid-cols-[1fr_auto_1fr] xl:text-sm">
                            <div class="flex flex-wrap items-center justify-center gap-x-4 gap-y-1 xl:justify-start">
                                {(!view_only)
                                    .then(|| {
                                        view! {
                                            <MenuButton
                                                on:click=move |_| {
                                                    skill_masteries
                                                        .update(|skill_masteries| {
                                                            if let Some(mastery) = skill_masteries
                                                                .masteries
                                                                .get_mut(&skill_id_for_reset)
                                                            {
                                                                mastery.upgrades_bought.clear();
                                                            }
                                                        });
                                                }
                                                disabled=Signal::derive(move || spent_points == 0)
                                            >
                                                "Reset"
                                            </MenuButton>
                                        }
                                    })}


                                <div class="flex gap-1">
                                    <span class="font-bold text-zinc-100">{mastery_level.saturating_sub(spent_points)}</span>
                                    <span class="text-zinc-600">"/"</span>
                                    <span class="font-bold text-zinc-500">{mastery_level}</span>
                                    <span class="font-semibold text-zinc-500">" Remaining Points"</span>
                                </div>

                            </div>
                            <div class="flex justify-center">
                                <CardInsetTitle separator=false>"Mastery Upgrades"</CardInsetTitle>
                            </div>
                            <div />
                        </div>

                        <div class="grid grid-cols-1 gap-2 xl:grid-cols-2">
                            <For
                                each=move || upgrades.clone().into_iter()
                                key=|(upgrade_id, _)| upgrade_id.clone()
                                let:((upgrade_id, upgrade_specs))
                            >
                                <MasteryUpgradeRow
                                    skill_id=skill_id.clone()
                                    upgrade_id
                                    upgrade_specs
                                    skill_masteries
                                    view_only
                                />
                            </For>
                        </div>
                    </section>
                }
                    .into_any()
            }}
        </div>
    }
}

#[component]
fn MasteryUpgradeRow(
    skill_id: String,
    upgrade_id: String,
    upgrade_specs: SkillMasteryUpgrade,
    skill_masteries: RwSignal<PlayerSkillMasteries>,
    view_only: bool,
) -> impl IntoView {
    let data_context = expect_context::<DataContext>();
    let events_context: EventsContext = expect_context();
    let upgrade_specs = Arc::new(upgrade_specs);

    let upgrade_level = Memo::new({
        let skill_id = skill_id.clone();
        let upgrade_id = upgrade_id.clone();
        move |_| {
            skill_masteries.with(|skill_masteries| {
                skill_masteries
                    .masteries
                    .get(&skill_id)
                    .and_then(|mastery| mastery.upgrades_bought.get(&upgrade_id))
                    .copied()
                    .unwrap_or_default()
            })
        }
    });

    let available_points = Memo::new({
        let skill_id = skill_id.clone();
        move |_| {
            skill_masteries.with(|skill_masteries| {
                let Some(mastery) = skill_masteries.masteries.get(&skill_id) else {
                    return 0;
                };
                let Some(mastery_specs) = data_context
                    .skill_mastery_specs
                    .read()
                    .get(&skill_id)
                    .cloned()
                else {
                    return 0;
                };
                mastery
                    .level()
                    .min(mastery_specs.max_level)
                    .saturating_sub(spent_mastery_points(mastery, &mastery_specs.upgrades))
            })
        }
    });

    let next_cost = Signal::derive({
        let upgrade_specs = upgrade_specs.clone();
        move || {
            upgrade_specs
                .compute_cost(upgrade_level.get().saturating_add(1))
                .saturating_sub(upgrade_specs.compute_cost(upgrade_level.get()))
        }
    });
    let can_add = Signal::derive({
        let upgrade_specs = upgrade_specs.clone();
        move || {
            !view_only
                && upgrade_level.get() < upgrade_specs.max_level
                && available_points.get() >= next_cost.get()
        }
    });
    let can_remove = Signal::derive(move || !view_only && upgrade_level.get() > 0);

    let add_point = {
        let skill_id = skill_id.clone();
        let upgrade_id = upgrade_id.clone();
        let upgrade_specs = upgrade_specs.clone();
        move |_| {
            let max_steps = if events_context.key_pressed(Key::Ctrl) {
                10
            } else {
                1
            };

            skill_masteries.update(|skill_masteries| {
                let Some(mastery) = skill_masteries.masteries.get_mut(&skill_id) else {
                    return;
                };
                let Some(mastery_specs) = data_context
                    .skill_mastery_specs
                    .read()
                    .get(&skill_id)
                    .cloned()
                else {
                    return;
                };

                for _ in 0..max_steps {
                    let current_level = mastery
                        .upgrades_bought
                        .get(&upgrade_id)
                        .copied()
                        .unwrap_or_default();
                    if current_level >= upgrade_specs.max_level {
                        break;
                    }

                    let spent_points = spent_mastery_points(mastery, &mastery_specs.upgrades);
                    let available_points = mastery
                        .level()
                        .min(mastery_specs.max_level)
                        .saturating_sub(spent_points);
                    let cost = upgrade_specs
                        .compute_cost(current_level.saturating_add(1))
                        .saturating_sub(upgrade_specs.compute_cost(current_level));

                    if available_points < cost {
                        break;
                    }

                    *mastery
                        .upgrades_bought
                        .entry(upgrade_id.clone())
                        .or_default() += 1;
                }
            });
        }
    };
    let remove_point = {
        let skill_id = skill_id.clone();
        let upgrade_id = upgrade_id.clone();
        move |_| {
            let amount = if events_context.key_pressed(Key::Ctrl) {
                10
            } else {
                1
            };

            skill_masteries.update(|skill_masteries| {
                let Some(mastery) = skill_masteries.masteries.get_mut(&skill_id) else {
                    return;
                };
                let Some(level) = mastery.upgrades_bought.get_mut(&upgrade_id) else {
                    return;
                };

                *level = level.saturating_sub(amount);
                if *level == 0 {
                    mastery.upgrades_bought.remove(&upgrade_id);
                }
            });
        }
    };

    let title = upgrade_specs.title.clone();

    view! {
        <MenuListRow class="min-w-0 overflow-hidden">
            <div class=if view_only {
                "grid grid-cols-1 gap-2 p-2 xl:p-3"
            } else {
                "grid grid-cols-[minmax(0,1fr)_auto] gap-2 p-2 xl:p-3"
            }>
                <div class="flex min-w-0 flex-col gap-2">
                    <div class="flex min-w-0 items-center justify-between gap-2">
                        <span class="text-shadow-lg/30 shadow-gray-950 text-amber-300 font-semibold text-sm xl:text-base">
                            {title}
                        </span>
                        <div class="flex shrink-0 items-center gap-3 text-xs xl:text-sm text-zinc-400">
                            {(next_cost.get() > 0 && upgrade_level.get() < upgrade_specs.max_level)
                                .then(|| {
                                    view! {
                                        <div>
                                            "Cost "
                                            <span class="font-bold text-zinc-100">
                                                {move || next_cost.get()}
                                            </span>
                                        </div>
                                    }
                                })}
                            <div>
                                "Mastery Level "
                                <span class="font-bold text-zinc-100">
                                    {move || upgrade_level.get()}
                                </span> <span class="text-zinc-600">" / "</span>
                                <span class="font-bold text-zinc-500">
                                    {upgrade_specs.max_level}
                                </span>
                            </div>
                        </div>
                    </div>

                    <div class="grid grid-cols-1 2xl:grid-cols-2 gap-2">
                        <div class="min-w-0 rounded-[7px] border border-black/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.02),transparent),linear-gradient(180deg,rgba(15,15,19,1),rgba(9,9,12,1))] p-2 shadow-[inset_0_1px_0_rgba(255,255,255,0.04),inset_0_-1px_0_rgba(0,0,0,0.45)]">
                            <div class="text-xs text-zinc-400 mb-1">"Current"</div>
                            <UpgradeEffectDescription
                                upgrade_specs=upgrade_specs.clone()
                                upgrade_level=Signal::derive(move || upgrade_level.get())
                            />
                        </div>

                        <div class="min-w-0 rounded-[7px] border border-black/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.02),transparent),linear-gradient(180deg,rgba(15,15,19,1),rgba(9,9,12,1))] p-2 shadow-[inset_0_1px_0_rgba(255,255,255,0.04),inset_0_-1px_0_rgba(0,0,0,0.45)]">
                            <div class="text-xs text-zinc-400 mb-1">"Next"</div>

                            <UpgradeEffectDescription
                                upgrade_specs=upgrade_specs.clone()
                                upgrade_level=Signal::derive(move || {
                                    upgrade_level.get().saturating_add(1)
                                })
                            />

                        </div>
                    </div>
                </div>
                {(!view_only)
                    .then(|| {
                        view! {
                            <div class="flex w-9 flex-col items-center justify-center gap-1 ml-2">
                                <MenuButton
                                    on:click=add_point
                                    disabled=Signal::derive(move || !can_add.get())
                                    class="w-9 px-0"
                                >
                                    "+"
                                </MenuButton>
                                <div class="min-w-9 text-center text-sm xl:text-base font-bold text-zinc-100 font-number">
                                    {move || upgrade_level.get()}
                                </div>
                                <MenuButton
                                    on:click=remove_point
                                    disabled=Signal::derive(move || !can_remove.get())
                                    class="w-9 px-0"
                                >
                                    "-"
                                </MenuButton>
                            </div>
                        }
                    })}
            </div>
        </MenuListRow>
    }
}

#[component]
fn UpgradeEffectDescription(
    upgrade_specs: Arc<SkillMasteryUpgrade>,
    upgrade_level: Signal<u16>,
) -> impl IntoView {
    view! {
        <ul class="text-xs xl:text-sm text-amber-100 break-words">
            {move || {
                let upgrade_level = upgrade_level.get();
                if upgrade_level > upgrade_specs.max_level {
                    view! { <li class="text-zinc-500">"Max Level"</li> }.into_any()
                } else if upgrade_level == 0 {
                    view! { <li class="text-zinc-500">"No effect"</li> }.into_any()
                } else {
                    let stat_effects: Vec<_> = upgrade_specs
                        .effects
                        .iter()
                        .filter_map(|effect| effect.compute_stat_effect(upgrade_level))
                        .collect();
                    view! {
                        {effects_tooltip::formatted_effects_list(stat_effects)}
                        {upgrade_specs
                            .effects
                            .iter()
                            .filter_map(|upgrade_effect| format_mastery_upgrade_effect(
                                upgrade_effect,
                                upgrade_level,
                            ))
                            .collect::<Vec<_>>()}
                    }
                        .into_any()
                }
            }}
        </ul>
    }
}

fn format_mastery_upgrade_effect(
    upgrade_effect: &SkillMasteryUpgradeEffect,
    _upgrade_level: u16,
) -> Option<impl IntoView> {
    match &upgrade_effect.effect_type {
        SkillMasteryUpgradeEffectType::StatEffect { .. } => None,
        SkillMasteryUpgradeEffectType::SkillEffect {
            skill_effect,
            target_index: _,
        } => Some(skill_tooltip::format_skill_effect(
            skill_effect.clone(),
            false,
            None,
            None,
            None,
            None,
        )),
    }
}

fn spent_mastery_points(
    mastery: &SkillMasteryState,
    upgrades: &indexmap::IndexMap<String, SkillMasteryUpgrade>,
) -> u16 {
    mastery
        .upgrades_bought
        .iter()
        .filter_map(|(upgrade_id, upgrade_level)| {
            upgrades
                .get(upgrade_id)
                .map(|upgrade| upgrade.compute_cost(*upgrade_level))
        })
        .fold(0u16, u16::saturating_add)
}

fn skill_type_title(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "Attacks",
        SkillType::Spell => "Spells",
        SkillType::Curse => "Curses",
        SkillType::Blessing => "Blessings",
        SkillType::Other => "Others",
    }
}

fn skill_type_title_color(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "text-red-300",
        SkillType::Spell => "text-sky-300",
        SkillType::Curse => "text-purple-300",
        SkillType::Blessing => "text-amber-200",
        SkillType::Other => "text-slate-300",
    }
}

fn skill_type_glow(skill_type: SkillType) -> &'static str {
    match skill_type {
        SkillType::Attack => "via-red-400/70",
        SkillType::Spell => "via-sky-400/70",
        SkillType::Curse => "via-purple-400/70",
        SkillType::Blessing => "via-amber-300/75",
        SkillType::Other => "via-slate-300/60",
    }
}
