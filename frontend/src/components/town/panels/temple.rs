use std::sync::Arc;

use leptos::{html::*, prelude::*, task::spawn_local};

use shared::{
    data::temple::{BenedictionEffect, BenedictionSpecs, BenedictionsCategory, PlayerBenedictions},
    http::client::BuyBenedictionsRequest,
};

use crate::components::{
    backend_client::BackendClient,
    data_context::DataContext,
    events::{EventsContext, Key},
    shared::{
        resources::{GoldCounter, GoldIcon},
        tooltips::effects_tooltip,
    },
    town::TownContext,
    ui::{
        TitleSeparator,
        buttons::MenuButton,
        card::{CardHeader, CardInset, CardInsetTitle, MenuCard},
        list_row::MenuListRow,
        menu_panel::MenuPanel,
        number::format_number,
        toast::*,
    },
};

#[component]
pub fn TemplePanel(
    open: RwSignal<bool>,
    #[prop(default = false)] view_only: bool,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let cost = RwSignal::new(0.0);
    let player_benedictions = RwSignal::new(PlayerBenedictions::default());

    let reset = move || {
        cost.set(0.0);
        player_benedictions.set(town_context.player_benedictions.get_untracked());
    };

    Effect::new(move || {
        if open.get() {
            reset();
        }
    });

    view! {
        <MenuPanel open=open>
            <MenuCard class="h-full">
                <CardHeader title="Temple" on_close=move || open.set(false)>
                    {(!view_only)
                        .then(|| {
                            view! {
                                <div class="flex-1" />

                                <div class="flex h-full items-center gap-1 text-sm xl:text-base text-zinc-300 mb-1">
                                    "Benedictions Cost: " <GoldCounter value=cost w_full=true />
                                </div>

                                <div class="flex-1" />

                                <div class="flex  h-full items-center gap-2">
                                    <MenuButton
                                        on:click=move |_| reset()
                                        disabled=Signal::derive(move || cost.get() == 0.0)
                                    >
                                        "Cancel"
                                    </MenuButton>
                                    <ConfirmButton player_benedictions open />
                                </div>
                            }
                        })}
                </CardHeader>
                <CardInset>
                    <BenedictionsList player_benedictions cost view_only />
                </CardInset>
            </MenuCard>
        </MenuPanel>
    }
}

#[component]
fn ConfirmButton(
    player_benedictions: RwSignal<PlayerBenedictions>,
    // cost: RwSignal<f64>,
    open: RwSignal<bool>,
) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();

    let do_buy = Arc::new({
        let toaster = expect_context::<Toasts>();

        let character_id = town_context.character.read_untracked().character_id;
        move || {
            spawn_local({
                async move {
                    match backend
                        .post_buy_benedictions(&BuyBenedictionsRequest {
                            character_id,
                            player_benedictions: player_benedictions.get_untracked(),
                        })
                        .await
                    {
                        Ok(response) => {
                            town_context.character.set(response.character);
                            town_context
                                .player_benedictions
                                .set(response.player_benedictions);
                            open.set(false);
                        }
                        Err(e) => show_toast(
                            toaster,
                            format!("Failed to buy benedictions: {e}"),
                            ToastVariant::Error,
                        ),
                    }
                }
            });
        }
    });

    // let try_buy = {
    //     let confirm_context: ConfirmContext = expect_context();
    //     let settings_context: SettingsContext = expect_context();
    //     move |_| {
    //         let cost_str = format_number_without_context(
    //             cost.get(),
    //             settings_context.read_settings().scientific_notation,
    //         );
    //         (confirm_context.confirm)(
    //             format! {"Do you confirm buying Temple points for {} Gold?", cost_str },
    //             do_buy.clone(),
    //         );
    //     }
    // };

    let disabled = Signal::derive(move || {
        *town_context.player_benedictions.read() == *player_benedictions.read()
    });

    view! {
        <MenuButton on:click=move |_| do_buy() disabled=disabled>
            "Confirm"
        </MenuButton>
    }
}

#[component]
fn BenedictionsList(
    player_benedictions: RwSignal<PlayerBenedictions>,
    cost: RwSignal<f64>,
    view_only: bool,
) -> impl IntoView {
    let data_context = expect_context::<DataContext>();

    view! {
        <div class="grid w-full grid-cols-1 items-start gap-x-6 gap-y-12 xl:grid-cols-2">
            <For
                each=move || data_context.benedictions_specs.get().into_iter()
                key=|(category_id, _)| category_id.clone()
                let:((category_id, category_specs))
            >
                <BenedictionCategorySection
                    category_id
                    category_specs
                    player_benedictions
                    cost
                    view_only
                />
            </For>
        </div>
    }
}

#[component]
fn BenedictionCategorySection(
    category_id: String,
    category_specs: BenedictionsCategory,
    player_benedictions: RwSignal<PlayerBenedictions>,
    cost: RwSignal<f64>,
    view_only: bool,
) -> impl IntoView {
    let category_title = category_specs.title.clone();
    let max_level_text = category_specs.max_upgrade_level.to_string();
    let max_upgrade_level = category_specs.max_upgrade_level;
    let price = Memo::new({
        let category_specs = category_specs.clone();
        let category_id = category_id.clone();
        move |_| {
            let upgrade_level = player_benedictions.with(|player_benedictions| {
                player_benedictions
                    .categories
                    .get(&category_id)
                    .map(|category| category.upgrade_level)
                    .unwrap_or_default()
            });
            category_specs.compute_price(upgrade_level)
        }
    });
    let bought_points = Memo::new({
        let category_id = category_id.clone();
        move |_| {
            player_benedictions.with(|player_benedictions| {
                player_benedictions
                    .categories
                    .get(&category_id)
                    .map(|category| category.upgrade_level)
                    .unwrap_or_default()
            })
        }
    });
    let allocated_points: Memo<u64> = Memo::new({
        let category_id = category_id.clone();
        move |_| {
            player_benedictions.with(|player_benedictions| {
                player_benedictions
                    .categories
                    .get(&category_id)
                    .map(|category| category.purchased_benedictions.values().sum())
                    .unwrap_or_default()
            })
        }
    });
    let max_level_reached = Signal::derive(move || bought_points.get() >= max_upgrade_level);
    let reset_disabled = Signal::derive(move || allocated_points.get() == 0);

    let reset_category = {
        let category_id = category_id.clone();
        move |_| {
            player_benedictions.update(|player_benedictions| {
                if let Some(category) = player_benedictions.categories.get_mut(&category_id) {
                    category.purchased_benedictions.clear();
                }
            });
        }
    };
    let benedictions = category_specs.benedictions.clone();
    let category_specs_for_rows = category_specs.clone();

    view! {
        <section class="w-full min-w-0">
            <TitleSeparator />

            <div class="mb-2 mt-1 grid grid-cols-3 gap-2 px-1 text-xs xl:text-sm">
                <div class="flex flex-wrap items-center justify-center gap-x-4 gap-y-1 justify-start">
                    {(!view_only)
                        .then(|| {
                            view! {
                                <MenuButton on:click=reset_category disabled=reset_disabled>
                                    "Reset"
                                </MenuButton>
                            }
                        })}
                    <div class="flex gap-1">
                        <span class="font-bold text-zinc-100">{move || bought_points.get().saturating_sub(allocated_points.get())}</span>
                        <span class="text-zinc-400">"/"</span>
                        <span class="font-bold text-zinc-400">{move || bought_points.get()}</span>
                        <span class="font-semibold text-zinc-400">" Available"</span>
                    </div>

                </div>

                <div>
                    <CardInsetTitle separator=false>{category_title}</CardInsetTitle>
                </div>

                <div class="flex flex-nowrap items-center justify-end gap-x-2 whitespace-nowrap">

                    <div class="flex gap-1">
                        <span class="font-semibold text-zinc-400">"Prayed"</span>
                        <span class="font-bold text-zinc-100">{move || bought_points.get()}</span>
                        <span class="text-zinc-400">"/"</span>
                        <span class="font-bold text-zinc-400">{max_level_text}</span>
                    </div>

                    {(!view_only)
                        .then(|| {
                            view! {
                                <div class="flex items-center gap-1">
                                    {move || {
                                        if max_level_reached.get() {
                                            view! {
                                                <span class="font-number font-semibold text-amber-200">
                                                    "Max"
                                                </span>
                                                <GoldIcon />
                                            }
                                                .into_any()
                                        } else {
                                            view! {
                                                <GoldCounter
                                                    value=Signal::derive(move || price.get())
                                                    w_full=true
                                                />
                                            }
                                                .into_any()
                                        }
                                    }}
                                </div>
                            }
                        })}
                </div>
            </div>

            <div class="grid grid-cols-1 gap-2 items-start">
                <For
                    each=move || benedictions.clone().into_iter()
                    key=|(benediction_id, _)| benediction_id.clone()
                    let:((benediction_id, benediction_specs))
                >
                    <BenedictionRow
                        category_id=category_id.clone()
                        benediction_id
                        benediction_specs
                        category_specs=category_specs_for_rows.clone()
                        player_benedictions
                        cost
                        view_only
                    />
                </For>
            </div>
        </section>
    }
}

#[component]
fn BenedictionRow(
    category_id: String,
    benediction_id: String,
    benediction_specs: BenedictionSpecs,
    category_specs: BenedictionsCategory,
    player_benedictions: RwSignal<PlayerBenedictions>,
    cost: RwSignal<f64>,
    view_only: bool,
) -> impl IntoView {
    let events_context: EventsContext = expect_context();
    let town_context = expect_context::<TownContext>();
    let max_upgrade_level = category_specs.max_upgrade_level;

    let upgrade_level = Memo::new({
        let category_id = category_id.clone();
        let benediction_id = benediction_id.clone();
        move |_| {
            player_benedictions.with(|player_benedictions| {
                player_benedictions
                    .categories
                    .get(&category_id)
                    .and_then(|category| category.purchased_benedictions.get(&benediction_id))
                    .copied()
                    .unwrap_or_default()
            })
        }
    });
    let available_points = Memo::new({
        let category_id = category_id.clone();
        move |_| {
            player_benedictions.with(|player_benedictions| {
                let Some(category) = player_benedictions.categories.get(&category_id) else {
                    return 0;
                };

                let allocated_points = category.purchased_benedictions.values().sum::<u64>();
                category.upgrade_level.saturating_sub(allocated_points)
            })
        }
    });
    let can_buy_point = Memo::new({
        let category_id = category_id.clone();
        let category_specs = category_specs.clone();
        move |_| {
            let bought_points = player_benedictions.with(|player_benedictions| {
                player_benedictions
                    .categories
                    .get(&category_id)
                    .map(|category| category.upgrade_level)
                    .unwrap_or_default()
            });
            let next_price = category_specs.compute_price(bought_points);

            bought_points < max_upgrade_level
                && cost.get() + next_price <= town_context.character.read().resource_gold
        }
    });
    let can_add = Signal::derive(move || {
        !view_only
            && upgrade_level.get() < max_upgrade_level
            && (available_points.get() > 0 || can_buy_point.get())
    });
    let can_remove = Signal::derive(move || !view_only && upgrade_level.get() > 0);

    let add_point = {
        let category_id = category_id.clone();
        let benediction_id = benediction_id.clone();
        let category_specs = category_specs.clone();
        move |_| {
            let requested_amount = if events_context.key_pressed(Key::Ctrl) {
                10
            } else {
                1
            };

            let player_benedictions_snapshot = player_benedictions.get_untracked();
            let category = player_benedictions_snapshot.categories.get(&category_id);
            let bought_points = category
                .map(|category| category.upgrade_level)
                .unwrap_or_default();
            let allocated_points = category
                .map(|category| category.purchased_benedictions.values().sum::<u64>())
                .unwrap_or_default();
            let benediction_level = category
                .and_then(|category| category.purchased_benedictions.get(&benediction_id))
                .copied()
                .unwrap_or_default();

            let amount = requested_amount.min(max_upgrade_level.saturating_sub(benediction_level));
            let available_points = bought_points.saturating_sub(allocated_points);
            let available_points_to_allocate = amount.min(available_points);
            let points_to_buy = amount.saturating_sub(available_points_to_allocate);
            let available_gold =
                town_context.character.read_untracked().resource_gold - cost.get_untracked();

            let mut bought_points_to_allocate = 0;
            let mut purchase_cost = 0.0;
            while bought_points_to_allocate < points_to_buy
                && bought_points + bought_points_to_allocate < max_upgrade_level
            {
                let next_price =
                    category_specs.compute_price(bought_points + bought_points_to_allocate);
                if purchase_cost + next_price > available_gold {
                    break;
                }

                purchase_cost += next_price;
                bought_points_to_allocate += 1;
            }

            let amount_to_allocate = available_points_to_allocate + bought_points_to_allocate;
            if amount_to_allocate == 0 {
                return;
            }

            cost.update(|cost| *cost += purchase_cost);
            player_benedictions.update(|player_benedictions| {
                let category = player_benedictions
                    .categories
                    .entry(category_id.clone())
                    .or_default();
                category.upgrade_level += bought_points_to_allocate;
                *category
                    .purchased_benedictions
                    .entry(benediction_id.clone())
                    .or_default() += amount_to_allocate;
            });
        }
    };
    let remove_point = {
        let category_id = category_id.clone();
        let benediction_id = benediction_id.clone();
        let category_specs = category_specs.clone();
        move |_| {
            let requested_amount = if events_context.key_pressed(Key::Ctrl) {
                10
            } else {
                1
            };

            let current_benedictions = player_benedictions.get_untracked();
            let Some(current_category) = current_benedictions.categories.get(&category_id) else {
                return;
            };
            let benediction_level = current_category
                .purchased_benedictions
                .get(&benediction_id)
                .copied()
                .unwrap_or_default();
            let amount_to_remove = requested_amount.min(benediction_level);
            if amount_to_remove == 0 {
                return;
            }

            let confirmed_bought_points = town_context
                .player_benedictions
                .get_untracked()
                .categories
                .get(&category_id)
                .map(|category| category.upgrade_level)
                .unwrap_or_default();
            let refundable_points = current_category
                .upgrade_level
                .saturating_sub(confirmed_bought_points);
            let points_to_refund = amount_to_remove.min(refundable_points);
            let refund = (0..points_to_refund)
                .map(|offset| {
                    category_specs.compute_price(current_category.upgrade_level - offset - 1)
                })
                .sum::<f64>();

            player_benedictions.update(|player_benedictions| {
                let Some(category) = player_benedictions.categories.get_mut(&category_id) else {
                    return;
                };
                let Some(upgrade_level) = category.purchased_benedictions.get_mut(&benediction_id)
                else {
                    return;
                };

                *upgrade_level = upgrade_level.saturating_sub(amount_to_remove);
                if *upgrade_level == 0 {
                    category.purchased_benedictions.remove(&benediction_id);
                }
                category.upgrade_level = category.upgrade_level.saturating_sub(points_to_refund);
            });
            cost.update(|cost| *cost = (*cost - refund).max(0.0));
        }
    };

    let benediction_title = format_benediction_title(&benediction_specs.effect);

    view! {
        <MenuListRow class="min-w-0 overflow-hidden">
            <div class=if view_only {
                "grid grid-cols-1 gap-2 p-2 xl:p-3"
            } else {
                "grid grid-cols-[minmax(0,1fr)_auto] gap-2 p-2 xl:p-3"
            }>
                <div class="flex min-w-0 flex-col gap-2">
                    <div class="flex min-w-0 items-center justify-between gap-2">
                        <span class="
                        text-shadow-lg/30 shadow-gray-950 text-amber-300 font-semibold
                        text-sm xl:text-base
                        ">{benediction_title}</span>

                        {view_only
                            .then(|| {
                                view! {
                                    <div class="shrink-0 text-xs xl:text-sm text-zinc-400">
                                        "Level "
                                        <span class="font-bold text-zinc-100">
                                            {move || upgrade_level.get()}
                                        </span>
                                    </div>
                                }
                            })}
                    </div>

                    <div class="grid grid-cols-1 2xl:grid-cols-2 gap-2">
                        <div class="min-w-0 rounded-[7px] border border-black/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.02),transparent),linear-gradient(180deg,rgba(15,15,19,1),rgba(9,9,12,1))] p-2 shadow-[inset_0_1px_0_rgba(255,255,255,0.04),inset_0_-1px_0_rgba(0,0,0,0.45)]">
                            <div class="text-xs text-zinc-400 mb-1">"Current"</div>
                            <EffectDescription
                                benediction_specs=benediction_specs.clone()
                                upgrade_level=Signal::derive(move || upgrade_level.get())
                                max_upgrade_level
                            />
                        </div>

                        <div class="min-w-0 rounded-[7px] border border-black/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.02),transparent),linear-gradient(180deg,rgba(15,15,19,1),rgba(9,9,12,1))] p-2 shadow-[inset_0_1px_0_rgba(255,255,255,0.04),inset_0_-1px_0_rgba(0,0,0,0.45)]">
                            <div class="text-xs text-zinc-400 mb-1">"Next"</div>
                            <EffectDescription
                                benediction_specs=benediction_specs.clone()
                                upgrade_level=Signal::derive(move || {
                                    upgrade_level.get().saturating_add(1)
                                })
                                max_upgrade_level
                            />
                        </div>
                    </div>
                </div>
                {(!view_only)
                    .then(|| {
                        let bold = move || upgrade_level.get() > 0;
                        view! {
                            <div class="flex w-9 flex-col items-center justify-center gap-1 ml-2">
                                <MenuButton
                                    on:click=add_point
                                    disabled=Signal::derive(move || !can_add.get())
                                    class="w-9 px-0"
                                >
                                    "+"
                                </MenuButton>
                                <div class="min-w-9 text-center text-sm xl:text-base font-number">
                                    <span class=move || {
                                        if bold() {
                                            "font-bold text-zinc-100"
                                        } else {
                                            "text-zinc-400"
                                        }
                                    }>{move || upgrade_level.get()}</span>
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

pub fn format_benediction_title(benediction_effect: &BenedictionEffect) -> String {
    match benediction_effect {
        BenedictionEffect::StartingGold => "Starting Gold".to_string(),
        BenedictionEffect::StartingLevel => "Starting Level".to_string(),
        BenedictionEffect::SkillSlots => "Skill Slots".to_string(),
        BenedictionEffect::StatEffect { stat, .. } => {
            effects_tooltip::format_multiplier_stat_name(stat)
        }
    }
}

#[component]
pub fn EffectDescription(
    benediction_specs: BenedictionSpecs,
    upgrade_level: Signal<u64>,
    max_upgrade_level: u64,
) -> impl IntoView {
    let value = {
        let benediction_specs = benediction_specs.clone();
        move || benediction_specs.compute_value(upgrade_level.get())
    };

    view! {
        <ul class="text-xs xl:text-sm text-amber-100 break-words">
            {move || {
                if upgrade_level.get() > max_upgrade_level {
                    return view! { <li class="text-zinc-400">"Max Level"</li> }.into_any();
                }
                value()
                    .map(|value| match benediction_specs.effect.clone() {
                        BenedictionEffect::StartingGold => {

                            view! {
                                {effects_tooltip::effect_li(
                                    format!("+{} Starting Gold", format_number(value)),
                                )}
                            }
                                .into_any()
                        }
                        BenedictionEffect::StartingLevel => {
                            view! {
                                {effects_tooltip::effect_li(
                                    format!("+{:0} Starting Player Level", value),
                                )}
                            }
                                .into_any()
                        }
                        BenedictionEffect::SkillSlots => {
                            view! {
                                {effects_tooltip::effect_li(format!("+{:0} Skill Slots", value))}
                            }
                                .into_any()
                        }
                        BenedictionEffect::StatEffect { .. } => {
                            view! {
                                {benediction_specs
                                    .compute_stat_effect(upgrade_level.get())
                                    .map(|stat_effect| effects_tooltip::formatted_effects_list(
                                        [stat_effect].into(),
                                    ))}
                            }
                                .into_any()
                        }
                    })
                    .unwrap_or_else(|| {
                        view! { <li class="text-zinc-400">"No effect"</li> }.into_any()
                    })
            }}
        </ul>
    }
}
