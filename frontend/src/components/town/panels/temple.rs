use leptos::{html::*, prelude::*, task::spawn_local};

use std::sync::Arc;

use shared::{
    data::{
        stat_effect::Modifier,
        temple::{BenedictionSpecs, PlayerBenedictions},
    },
    http::client::BuyBenedictionsRequest,
};

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    shared::tooltips::effects_tooltip::formatted_effects_list,
    town::TownContext,
    ui::{
        buttons::{CloseButton, MenuButton},
        confirm::ConfirmContext,
        menu_panel::{MenuPanel, PanelTitle},
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
                        {(view_only)
                            .then(|| {
                                view! {
                                    <span class="text-sm xl:text-base text-gray-400">
                                        "Benedictions Cost: "
                                        <span class="text-amber-300">{cost}" Gold"</span>
                                    </span>

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

    view! {
        <div class="relative min-h-0 flex-1  overflow-y-auto bg-neutral-900 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)] flex flex-col gap-2">
            {town_context
                .benedictions_specs
                .get()
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
                .collect::<Vec<_>>()}
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
    let upgrade_level = Memo::new({
        move |_| {
            player_benedictions
                .read()
                .benedictions
                .get(&benediction_id)
                .map(|benediction_state| benediction_state.upgrade_level)
                .unwrap_or_default()
        }
    });

    let effect = move || benediction_specs.compute_effect(upgrade_level.get());

    view! {
        <div class="flex justify-between">
            <ul>{move || formatted_effects_list([effect()].into())}</ul>
            <MenuButton>"Buy"</MenuButton>
        </div>
    }
}
