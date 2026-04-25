use std::sync::Arc;

use leptos::{html::*, prelude::*, task::spawn_local};

use shared::{
    data::temple::{BenedictionEffect, BenedictionSpecs, BenedictionsCategory, PlayerBenedictions},
    http::client::BuyBenedictionsRequest,
};

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    settings::SettingsContext,
    shared::{resources::GoldIcon, tooltips::effects_tooltip},
    town::TownContext,
    ui::{
        buttons::{MenuButton, MenuButtonRed},
        card::{CardHeader, CardInset, CardInsetTitle, MenuCard},
        confirm::ConfirmContext,
        list_row::MenuListRow,
        menu_panel::MenuPanel,
        number::{format_number, format_number_without_context},
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
            <MenuCard class="h-full" gap=false>
                <CardHeader title="Temple" on_close=move || open.set(false)>
                    {(!view_only)
                        .then(|| {
                            view! {
                                <div class="flex-1" />

                                <div class="flex h-full items-center gap-1 text-sm xl:text-base text-gray-300 mb-1">
                                    "Benedictions Cost: "
                                    <span class="text-amber-200 font-bold font-number">
                                        {move || format_number(cost.get())}
                                    </span> <GoldIcon />
                                </div>

                                <div class="flex-1" />

                                <div class="flex items-center gap-2">
                                    <MenuButton
                                        on:click=move |_| reset()
                                        disabled=Signal::derive(move || cost.get() == 0.0)
                                    >
                                        "Cancel"
                                    </MenuButton>
                                    <ConfirmButton player_benedictions cost open />
                                </div>
                            }
                        })}
                </CardHeader>
                <CardInset class="min-h-0 flex-1" gap=false pad=false>
                    <BenedictionsList player_benedictions cost view_only />
                </CardInset>
            </MenuCard>
        </MenuPanel>
    }
}

#[component]
fn ConfirmButton(
    player_benedictions: RwSignal<PlayerBenedictions>,
    cost: RwSignal<f64>,
    open: RwSignal<bool>,
) -> impl IntoView {
    let do_buy = Arc::new({
        let backend = expect_context::<BackendClient>();
        let town_context = expect_context::<TownContext>();
        let auth_context = expect_context::<AuthContext>();
        let toaster = expect_context::<Toasts>();

        let character_id = town_context.character.read_untracked().character_id;
        move || {
            spawn_local({
                async move {
                    match backend
                        .post_buy_benedictions(
                            &auth_context.token(),
                            &BuyBenedictionsRequest {
                                character_id,
                                player_benedictions: player_benedictions.get_untracked(),
                            },
                        )
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

    let try_buy = {
        let confirm_context: ConfirmContext = expect_context();
        let settings_context: SettingsContext = expect_context();
        move |_| {
            let cost_str = format_number_without_context(
                cost.get(),
                settings_context.read_settings().scientific_notation,
            );
            (confirm_context.confirm)(
                format! {"Do you confirm buying Temple points for {} Gold?", cost_str },
                do_buy.clone(),
            );
        }
    };

    let disabled = Signal::derive(move || cost.get() == 0.0);

    view! {
        <MenuButton on:click=try_buy disabled=disabled>
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
    let town_context = expect_context::<TownContext>();

    let benediction_categories = Memo::new(move |_| {
        town_context.benedictions_specs.with(|specs| {
            let mut categories = specs
                .iter()
                .map(|(category_id, category_specs)| (category_id.clone(), category_specs.clone()))
                .collect::<Vec<_>>();

            categories.sort_by(|(a_id, a_specs), (b_id, b_specs)| {
                a_specs.title.cmp(&b_specs.title).then(a_id.cmp(b_id))
            });
            categories
        })
    });

    view! {
        <div class="w-full space-y-4 p-1 xl:p-3">
            {move || {
                benediction_categories
                    .get()
                    .into_iter()
                    .map(|(category_id, category_specs)| {
                        view! {
                            <BenedictionCategorySection
                                category_id
                                category_specs
                                player_benedictions
                                cost
                                view_only
                            />
                        }
                    })
                    .collect::<Vec<_>>()
            }}
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
    let town_context = expect_context::<TownContext>();

    let category_title = category_specs.title.clone();
    let max_level_text = category_specs
        .max_upgrade_level
        .map(|max_level| max_level.to_string())
        .unwrap_or_else(|| "No cap".to_string());
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
    let allocated_points = Memo::new({
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
    let available_points =
        Memo::new(move |_| bought_points.get().saturating_sub(allocated_points.get()));
    let max_level_reached = Signal::derive({
        let max_upgrade_level = category_specs.max_upgrade_level;
        move || {
            max_upgrade_level
                .map(|max_upgrade_level| bought_points.get() >= max_upgrade_level)
                .unwrap_or_default()
        }
    });
    let buy_disabled = Signal::derive(move || {
        max_level_reached.get()
            || cost.get() + price.get() > town_context.character.read().resource_gold
    });
    let reset_disabled = Signal::derive(move || allocated_points.get() == 0);

    let buy_point = {
        let category_id = category_id.clone();
        move |_| {
            if max_level_reached.get_untracked()
                || cost.get_untracked() + price.get_untracked()
                    > town_context.character.read_untracked().resource_gold
            {
                return;
            }

            cost.update(|cost| *cost += price.get_untracked());
            player_benedictions.update(|player_benedictions| {
                player_benedictions
                    .categories
                    .entry(category_id.clone())
                    .or_default()
                    .upgrade_level += 1;
            });
        }
    };

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

    let benedictions = category_specs
        .benedictions
        .iter()
        .map(|(benediction_id, benediction_specs)| {
            (benediction_id.clone(), benediction_specs.clone())
        })
        .collect::<Vec<_>>();
    // benedictions.sort_by(|(a_id, a_specs), (b_id, b_specs)| {
    //     a_specs.effect.cmp(&b_specs.effect).then(a_id.cmp(b_id))
    // });

    view! {
        <section class="w-full min-w-0">
            <CardInsetTitle>
                <div class="flex flex-wrap items-center justify-between gap-2">

                    <div class="flex flex-wrap items-center justify-end gap-x-2 gap-y-1 text-xs xl:text-sm tracking-normal font-sans">
                        <span class="text-zinc-400">
                            "Points "
                            <span class="font-bold text-amber-100">
                                {move || available_points.get()}
                            </span> <span class="text-zinc-500">"/"</span>
                            <span class="font-bold text-zinc-100">
                                {move || bought_points.get()}
                            </span>
                        </span>
                        <span class="text-zinc-600">"|"</span>
                        <span class="text-zinc-400">
                            "Max " <span class="font-semibold text-zinc-200">{max_level_text}</span>
                        </span>

                        {(!view_only)
                            .then(|| {
                                view! {
                                    <MenuButtonRed on:click=reset_category disabled=reset_disabled>
                                        "Reset"
                                    </MenuButtonRed>
                                }
                            })}
                    </div>

                    <span>{category_title}</span>

                    <div class="flex flex-wrap items-center justify-end gap-x-2 gap-y-1 text-xs xl:text-sm tracking-normal font-sans">

                        {(!view_only)
                            .then(|| {
                                view! {
                                    <MenuButton on:click=buy_point disabled=buy_disabled>
                                        <div class="flex items-center gap-1">
                                            {move || {
                                                if max_level_reached.get() {
                                                    view! { "Max" }.into_any()
                                                } else {
                                                    view! {
                                                        "Pray"
                                                        <span class="text-amber-200 font-bold font-number">
                                                            {format_number(price.get())}
                                                        </span>
                                                        <GoldIcon />
                                                    }
                                                        .into_any()
                                                }
                                            }}
                                        </div>
                                    </MenuButton>
                                }
                            })}
                    </div>
                </div>
            </CardInsetTitle>

            <div class="grid grid-cols-1 xl:grid-cols-2 gap-2 items-start">
                {benedictions
                    .into_iter()
                    .map(|(benediction_id, benediction_specs)| {
                        view! {
                            <BenedictionRow
                                category_id=category_id.clone()
                                benediction_id
                                benediction_specs
                                player_benedictions
                                view_only
                            />
                        }
                    })
                    .collect::<Vec<_>>()}
            </div>
        </section>
    }
}

#[component]
fn BenedictionRow(
    category_id: String,
    benediction_id: String,
    benediction_specs: BenedictionSpecs,
    player_benedictions: RwSignal<PlayerBenedictions>,
    view_only: bool,
) -> impl IntoView {
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
    let can_add = Signal::derive(move || !view_only && available_points.get() > 0);
    let can_remove = Signal::derive(move || !view_only && upgrade_level.get() > 0);

    let add_point = {
        let category_id = category_id.clone();
        let benediction_id = benediction_id.clone();
        move |_| {
            player_benedictions.update(|player_benedictions| {
                let category = player_benedictions
                    .categories
                    .entry(category_id.clone())
                    .or_default();
                let allocated_points = category.purchased_benedictions.values().sum::<u64>();

                if allocated_points < category.upgrade_level {
                    *category
                        .purchased_benedictions
                        .entry(benediction_id.clone())
                        .or_default() += 1;
                }
            });
        }
    };
    let remove_point = {
        let category_id = category_id.clone();
        let benediction_id = benediction_id.clone();
        move |_| {
            player_benedictions.update(|player_benedictions| {
                let Some(category) = player_benedictions.categories.get_mut(&category_id) else {
                    return;
                };
                let Some(upgrade_level) = category.purchased_benedictions.get_mut(&benediction_id)
                else {
                    return;
                };

                *upgrade_level = upgrade_level.saturating_sub(1);
                if *upgrade_level == 0 {
                    category.purchased_benedictions.remove(&benediction_id);
                }
            });
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
                        text-shadow-lg/100 shadow-gray-950 text-amber-200 font-semibold
                        text-sm xl:text-base font-display truncate
                        ">{benediction_title}</span>

                        {view_only
                            .then(|| {
                                view! {
                                    <div class="shrink-0 text-xs xl:text-sm text-gray-400">
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
                            <div class="text-xs text-gray-400 mb-1">"Current"</div>
                            <EffectDescription
                                benediction_specs=benediction_specs.clone()
                                upgrade_level=Signal::derive(move || upgrade_level.get())
                            />
                        </div>

                        <div class="min-w-0 rounded-[7px] border border-black/70 bg-[linear-gradient(180deg,rgba(255,255,255,0.02),transparent),linear-gradient(180deg,rgba(15,15,19,1),rgba(9,9,12,1))] p-2 shadow-[inset_0_1px_0_rgba(255,255,255,0.04),inset_0_-1px_0_rgba(0,0,0,0.45)]">
                            <div class="text-xs text-gray-400 mb-1">"Next"</div>
                            <EffectDescription
                                benediction_specs=benediction_specs.clone()
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
                                <div class="min-w-9 text-center text-sm xl:text-base font-bold text-amber-100 font-number">
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

pub fn format_benediction_title(benediction_effect: &BenedictionEffect) -> String {
    match benediction_effect {
        BenedictionEffect::StartingGold => "Starting Gold".to_string(),
        BenedictionEffect::StartingLevel => "Starting Level".to_string(),
        BenedictionEffect::StatEffect { stat, .. } => {
            effects_tooltip::format_multiplier_stat_name(stat)
        }
    }
}

#[component]
pub fn EffectDescription(
    benediction_specs: BenedictionSpecs,
    upgrade_level: Signal<u64>,
) -> impl IntoView {
    let value = {
        let benediction_specs = benediction_specs.clone();
        move || benediction_specs.compute_value(upgrade_level.get())
    };

    view! {
        <ul class="text-xs xl:text-sm text-amber-100 break-words">
            {move || {
                value()
                    .map(|value| match benediction_specs.effect.clone() {
                        BenedictionEffect::StartingGold => {
                            view! {
                                {effects_tooltip::effect_li(format!("+{:0} Starting Gold", value))}
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
                        view! { <li class="text-zinc-500">"No effect"</li> }.into_any()
                    })
            }}
        </ul>
    }
}
