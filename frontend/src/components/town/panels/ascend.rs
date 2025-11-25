use leptos::{html::*, prelude::*, task::spawn_local};

use std::sync::Arc;

use shared::{
    data::passive::{PassiveNodeId, PassiveNodeSpecs, PassivesTreeAscension},
    http::client::AscendPassivesRequest,
};

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    game::panels::passives::{Connection, MetaStatus, Node, NodeStatus, PurchaseStatus},
    town::TownContext,
    ui::{
        buttons::{CloseButton, MenuButton},
        confirm::ConfirmContext,
        menu_panel::{MenuPanel, PanelTitle},
        pannable::Pannable,
        toast::*,
    },
};

#[component]
pub fn AscendPanel(
    open: RwSignal<bool>,
    #[prop(default = false)] view_only: bool,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let ascension_cost = RwSignal::new(0.0);
    let passives_tree_ascension = RwSignal::new(PassivesTreeAscension::default());

    let reset = move || {
        ascension_cost.set(0.0);
        passives_tree_ascension.set(town_context.passives_tree_ascension.get_untracked());
    };

    let has_changed = Memo::new(move |_| {
        town_context
            .passives_tree_ascension
            .with(|base_ascension| !passives_tree_ascension.read().eq(base_ascension))
    });

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
                        {if view_only {
                            view! { <PanelTitle>"Ascended Passive Skills"</PanelTitle> }.into_any()
                        } else {
                            view! {
                                <PanelTitle>"Ascend Passive Skills"</PanelTitle>

                                <span class="text-sm xl:text-base text-gray-400">
                                    "Ascension Cost: "
                                    <span class="text-cyan-300">
                                        {ascension_cost}" Power Shards"
                                    </span>
                                </span>

                                <div class="flex items-center gap-2">
                                    <MenuButton
                                        on:click=move |_| reset()
                                        disabled=Signal::derive(move || !has_changed.get())
                                    >
                                        "Cancel"
                                    </MenuButton>
                                    <ConfirmButton
                                        passives_tree_ascension
                                        ascension_cost
                                        has_changed
                                        open
                                    />
                                </div>
                            }
                                .into_any()
                        }} <CloseButton on:click=move |_| open.set(false) />
                    </div>

                    <PassiveSkillTree passives_tree_ascension ascension_cost view_only />

                    {(!view_only)
                        .then(|| {
                            view! {
                                <div class="px-2 xl:px-4 relative z-10 flex items-center justify-between">
                                    <div class="flex items-center gap-2">
                                        <ResetButton passives_tree_ascension ascension_cost />
                                    </div>
                                </div>
                            }
                        })}

                </div>
            </div>
        </MenuPanel>
    }
}

#[component]
fn ConfirmButton(
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    ascension_cost: RwSignal<f64>,
    has_changed: Memo<bool>,
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
                        .post_ascend_passives(
                            &auth_context.token(),
                            &AscendPassivesRequest {
                                character_id,
                                passives_tree_ascension: passives_tree_ascension.get_untracked(),
                            },
                        )
                        .await
                    {
                        Ok(response) => {
                            town_context.character.set(response.character);
                            town_context.passives_tree_ascension.set(response.ascension);
                            open.set(false);
                        }
                        Err(e) => show_toast(
                            toaster,
                            format!("failed to ascend: {e}"),
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
                format! {"Do you confirm Ascension for {} Power Shards?",ascension_cost.get() },
                do_ascend.clone(),
            );
        }
    };

    let disabled = Signal::derive(move || !has_changed.get());

    view! {
        <MenuButton on:click=try_ascend disabled=disabled>
            "Confirm Ascension"
        </MenuButton>
    }
}

#[component]
fn ResetButton(
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    ascension_cost: RwSignal<f64>,
) -> impl IntoView {
    let do_reset = Arc::new({
        let backend = expect_context::<BackendClient>();
        let town_context = expect_context::<TownContext>();
        let auth_context = expect_context::<AuthContext>();
        let toaster = expect_context::<Toasts>();
        let character_id = town_context.character.read_untracked().character_id;
        move || {
            spawn_local({
                async move {
                    match backend
                        .post_ascend_passives(
                            &auth_context.token(),
                            &AscendPassivesRequest {
                                character_id,
                                passives_tree_ascension: PassivesTreeAscension::default(),
                            },
                        )
                        .await
                    {
                        Ok(response) => {
                            passives_tree_ascension.set(response.ascension.clone());
                            ascension_cost.set(0.0);
                            town_context.character.set(response.character);
                            town_context.passives_tree_ascension.set(response.ascension);
                        }
                        Err(e) => show_toast(
                            toaster,
                            format!("failed to refund: {e}"),
                            ToastVariant::Error,
                        ),
                    }
                }
            });
        }
    });

    let try_reset = {
        let confirm_context = expect_context::<ConfirmContext>();
        move |_| {
            (confirm_context.confirm)(
                "Do you confirm fully Refund Ascension and reclaim all Power Shards?".to_string(),
                do_reset.clone(),
            );
        }
    };

    view! { <MenuButton on:click=try_reset>"Refund Ascension"</MenuButton> }
}

#[component]
fn PassiveSkillTree(
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    ascension_cost: RwSignal<f64>,
    view_only: bool,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let points_available = Memo::new(move |_| {
        if view_only {
            0.0
        } else {
            (town_context.character.read().resource_shards - ascension_cost.get()).round()
        }
    });

    let nodes_specs = Arc::new(
        town_context
            .passives_tree_specs
            .read_untracked()
            .nodes
            .clone(),
    );

    // Fake amount of connections & ascended to have neatly rendered skill tree
    let amount_connections = Memo::new(|_| 0);
    // TODO: Should get actual levels?
    let node_levels = Memo::new(|_| (0, 0));

    view! {
        <Pannable>
            <For
                each=move || {
                    town_context.passives_tree_specs.read().connections.clone().into_iter()
                }
                key=|conn| (conn.from.clone(), conn.to.clone())
                let(conn)
            >
                <Connection
                    connection=conn
                    nodes_specs=nodes_specs.clone()
                    amount_connections
                    node_levels
                />
            </For>
            <For
                each=move || { town_context.passives_tree_specs.read().nodes.clone().into_iter() }
                key=|(id, _)| id.clone()
                let((id, node))
            >
                <AscendNode
                    node_id=id
                    node_specs=node
                    points_available
                    ascension_cost
                    passives_tree_ascension
                    view_only
                />
            </For>
        </Pannable>
    }
}

#[component]
fn AscendNode(
    node_id: PassiveNodeId,
    node_specs: PassiveNodeSpecs,
    points_available: Memo<f64>,
    ascension_cost: RwSignal<f64>,
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    view_only: bool,
) -> impl IntoView {
    let node_level = Memo::new({
        let node_id = node_id.clone();

        move |_| {
            passives_tree_ascension
                .read()
                .ascended_nodes
                .get(&node_id)
                .cloned()
                .unwrap_or_default()
        }
    });

    let max_upgrade_level = if node_specs.upgrade_effects.is_empty() {
        0
    } else {
        node_specs.max_upgrade_level.unwrap_or(u8::MAX)
    };

    let node_status = Memo::new({
        move |_| {
            let upgradable = max_upgrade_level > node_level.get();
            // let maxed = node_level.get() >= max_upgrade_level && node_level.get() > 0;

            let purchase_status =
            //  if maxed {
            //     PurchaseStatus::Inactive
            // } else 
            if view_only && node_level.get() > 0 {
                PurchaseStatus::Purchased
            } else if points_available.get() > 0.0
                && (upgradable || (node_specs.locked && node_level.get() == 0))
            {
                PurchaseStatus::Purchaseable
            } else {
                PurchaseStatus::Inactive
            };

            let meta_status = if node_level.get() > 0 {
                // if node_specs.locked && node_level.get() == 1 {
                //     MetaStatus::Normal
                // } else {
                //     MetaStatus::Ascended
                // }
                MetaStatus::Ascended
            } else if node_specs.locked {
                MetaStatus::Locked
            } else {
                MetaStatus::Normal
            };

            NodeStatus {
                purchase_status,
                meta_status,
            }
        }
    });

    let purchase = {
        let node_id = node_id.clone();
        move || {
            passives_tree_ascension.update(|passives_tree_ascension| {
                let entry = passives_tree_ascension
                    .ascended_nodes
                    .entry(node_id.clone())
                    .or_default();
                *entry = entry.saturating_add(1);
            });
            ascension_cost.update(|ascension_cost| *ascension_cost += 1.0); // TODO: Ascend cost?
        }
    };

    let refund = {
        let node_id = node_id.clone();
        move || {
            passives_tree_ascension.update(|passives_tree_ascension| {
                let entry = passives_tree_ascension
                    .ascended_nodes
                    .entry(node_id.clone())
                    .or_default();
                if *entry > 0 {
                    *entry = entry.saturating_sub(1);
                    ascension_cost.update(|ascension_cost| *ascension_cost -= 1.0);
                }
            });
        }
    };

    view! {
        <Node
            node_specs
            node_status
            node_level
            on_click=purchase
            on_right_click=refund
            show_upgrade=true
        />
    }
}
