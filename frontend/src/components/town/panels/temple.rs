use leptos::{html::*, prelude::*, task::spawn_local};

use std::sync::Arc;

use shared::{
    data::temple::{BenedictionSpecs, PlayerBenedictions},
    http::client::BuyBenedictionsRequest,
};

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    shared::{resources::GoldIcon, tooltips::effects_tooltip},
    town::TownContext,
    ui::{
        buttons::{CloseButton, MenuButton},
        confirm::ConfirmContext,
        menu_panel::{MenuPanel, PanelTitle},
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
    // Reset temporary ascension on opening
    Effect::new(move || {
        if open.get() {
            reset();
        }
    });

    view! {
        <MenuPanel open=open>
            <div class="w-full h-full">
                <div class="bg-zinc-800 rounded-md p-1 xl:p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col gap-1 xl:gap-2 max-h-full">
                    <div class="px-2 xl:px-4 flex items-center justify-between">
                        <PanelTitle>"Temple"</PanelTitle>
                        {(!view_only)
                            .then(|| {
                                view! {
                                    <div class="flex h-full items-center gap-1 text-sm xl:text-base text-gray-300 mb-1">
                                        "Benedictions Cost: "
                                        <span class="text-amber-200 font-bold font-number">
                                            {move || format_number(cost.get())}
                                        </span> <GoldIcon />
                                    </div>

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
                        <CloseButton on:click=move |_| open.set(false) />
                    </div>

                    <BenedictionsList player_benedictions cost view_only />
                </div>
            </div>
        </MenuPanel>
    }
}

#[component]
fn ConfirmButton(
    player_benedictions: RwSignal<PlayerBenedictions>,
    cost: RwSignal<f64>,
    open: RwSignal<bool>,
) -> impl IntoView {
    let do_ascend = Arc::new({
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
                            format!("failed to buy benedictions: {e}"),
                            ToastVariant::Error,
                        ),
                    }
                }
            });
        }
    });

    let try_ascend = {
        let confirm_context = expect_context::<ConfirmContext>();
        move |_| {
            (confirm_context.confirm)(
                format! {"Do you confirm buying Benedictions for {} Gold?",cost.get() },
                do_ascend.clone(),
            );
        }
    };

    let disabled = Signal::derive(move || cost.get() == 0.0);

    view! {
        <MenuButton on:click=try_ascend disabled=disabled>
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

    let benedictions_specs = move || {
        let mut benedictions_specs: Vec<_> =
            town_context.benedictions_specs.get().into_iter().collect();
        benedictions_specs
            .sort_by_key(|(_, specs)| (specs.effect.stat.clone(), specs.effect.modifier));
        benedictions_specs
    };

    view! {
        <div class="relative min-h-0 flex-1 overflow-y-auto
        bg-neutral-900 ring-1 ring-zinc-950 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)] 
        flex flex-col gap-2 p-1 xl:p-2">
            {move || {
                benedictions_specs()
                    .into_iter()
                    .map(|(benediction_id, benediction_specs)| {
                        view! {
                            <BenedictionRow
                                benediction_id
                                benediction_specs
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
fn BenedictionRow(
    benediction_id: String,
    benediction_specs: BenedictionSpecs,
    player_benedictions: RwSignal<PlayerBenedictions>,
    cost: RwSignal<f64>,
    view_only: bool,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let upgrade_level = Memo::new({
        let benediction_id = benediction_id.clone();
        move |_| {
            player_benedictions
                .read()
                .benedictions
                .get(&benediction_id)
                .map(|benediction_state| benediction_state.upgrade_level)
                .unwrap_or_default()
        }
    });

    let effect = {
        let benediction_specs = benediction_specs.clone();
        move || benediction_specs.compute_effect(upgrade_level.get())
    };
    let next_effect = {
        let benediction_specs = benediction_specs.clone();
        move || benediction_specs.compute_effect(upgrade_level.get() + 1)
    };
    let price = Memo::new({
        let benediction_specs = benediction_specs.clone();
        move |_| benediction_specs.compute_price(upgrade_level.get())
    });

    let max_level = {
        let benediction_specs = benediction_specs.clone();
        move || {
            benediction_specs
                .max_upgrade_level
                .map(|max_upgrade_level| max_upgrade_level <= upgrade_level.get())
                .unwrap_or_default()
        }
    };

    let disabled = Signal::derive(move || {
        max_level() || cost.get() + price.get() > town_context.character.read().resource_gold
    });

    let upgrade = {
        let benediction_id = benediction_id.clone();
        move |_| {
            cost.update(|cost| *cost += price.get_untracked());
            player_benedictions.update(|player_benedictions| {
                player_benedictions
                    .benedictions
                    .entry(benediction_id.clone())
                    .or_default()
                    .upgrade_level += 1;
            })
        }
    };

    view! {
        <div class="p-4 rounded-lg bg-zinc-800 border border-zinc-700
        shadow-inner flex flex-row gap-6 items-start
        transition-colors">

            <div class="flex flex-col flex-1 gap-1">

                <div class="flex items-center justify-between">
                    <div class="text-lg font-semibold text-amber-200 capitalize">
                        {effects_tooltip::format_multiplier_stat_name(
                            &benediction_specs.effect.stat,
                        )}
                    </div>

                    <div class="text-sm text-gray-400">
                        "Level " {upgrade_level.get()}
                        {benediction_specs
                            .max_upgrade_level
                            .map(|max_upgrade_level| format!("/{}", max_upgrade_level))}
                    </div>
                </div>

                <div class="grid grid-cols-2 gap-2 mt-1">

                    <div class="p-2 bg-zinc-900 rounded border border-zinc-700">
                        <div class="text-xs text-gray-400 mb-1">"Current"</div>
                        <ul class="text-sm text-amber-100">
                            {move || {
                                effect()
                                    .map(|effect| effects_tooltip::formatted_effects_list(
                                        [effect].into(),
                                    ))
                            }}
                        </ul>
                    </div>

                    <div class="p-2 bg-zinc-900 rounded border border-zinc-700">
                        <div class="text-xs text-gray-400 mb-1">"Next"</div>
                        <ul class="text-sm text-amber-100">
                            {move || {
                                next_effect()
                                    .map(|next_effect| effects_tooltip::formatted_effects_list(
                                        [next_effect].into(),
                                    ))
                            }}
                        </ul>
                    </div>

                </div>
            </div>

            {(!view_only)
                .then(|| {
                    view! {
                        <div class="flex flex-col h-full items-end justify-center min-w-[140px]">
                            {if max_level() {
                                view! {
                                    <span class="text-green-400 font-bold text-lg">"MAX"</span>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    <div class="flex items-center gap-1 text-sm xl:text-base text-gray-300 mb-1">
                                        <span class="text-amber-200 font-bold font-number">
                                            {move || format_number(price.get())}
                                        </span>
                                        <GoldIcon />
                                    </div>

                                    <MenuButton disabled class:h-full on:click=upgrade>
                                        "Upgrade"
                                    </MenuButton>
                                }
                                    .into_any()
                            }}

                        </div>
                    }
                })}

        </div>
    }
}
