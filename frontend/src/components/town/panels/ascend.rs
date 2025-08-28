use leptos::{html::*, prelude::*, task::spawn_local};

use std::sync::Arc;

use shared::{
    data::passive::{PassiveNodeId, PassiveNodeSpecs, PassivesTreeState},
    http::client::AscendPassivesRequest,
};

use crate::components::{
    backend_client::BackendClient,
    game::panels::passives::{Connection, MetaStatus, Node, NodeStatus, PurchaseStatus},
    town::TownContext,
    ui::{
        buttons::{CloseButton, MenuButton},
        confirm::ConfirmContext,
        menu_panel::MenuPanel,
        pannable::Pannable,
    },
};

#[component]
pub fn AscendPanel(open: RwSignal<bool>) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let ascension_cost = RwSignal::new(0.0);
    let passives_tree_state = RwSignal::new(PassivesTreeState::default());

    let reset = move || {
        ascension_cost.set(0.0);
        passives_tree_state.set(town_context.passives_tree_state.get_untracked());
    };
    // Reset temporary ascension on opening
    Effect::new(move || {
        if open.get() {
            reset();
        }
    });

    view! {
        <MenuPanel open=open>
            <div class="w-full p-4">
                <div class="bg-zinc-800 rounded-md p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col gap-2">
                    <div class="px-4 relative z-10 flex items-center justify-between">
                        <span class="text-shadow-md shadow-gray-950 text-amber-200 text-xl font-semibold">
                            "Passive Skills "
                        </span>

                        <span class="text-gray-300">
                            "Ascension Cost: "
                            <span class="text-cyan-200">{ascension_cost}" Power Shards"</span>
                        </span>

                        <div class="flex items-center gap-2">
                            <MenuButton
                                on:click=move |_| reset()
                                disabled=Signal::derive(move || ascension_cost.get() == 0.0)
                            >
                                "Cancel"
                            </MenuButton>
                            <ConfirmButton passives_tree_state ascension_cost open />
                            <CloseButton on:click=move |_| open.set(false) />
                        </div>
                    </div>

                    <PassiveSkillTree passives_tree_state ascension_cost />

                    <div class="px-4 relative z-10 flex items-center justify-between">

                        <div class="flex items-center gap-2">
                            <ResetButton />
                        </div>

                    </div>
                </div>
            </div>
        </MenuPanel>
    }
}

#[component]
fn ConfirmButton(
    passives_tree_state: RwSignal<PassivesTreeState>,
    ascension_cost: RwSignal<f64>,
    open: RwSignal<bool>,
) -> impl IntoView {
    let confirm_context = expect_context::<ConfirmContext>();

    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let ascend = Arc::new(move || {
        spawn_local({
            async move {
                // TODO:Toast error
                let _ = backend
                    .post_ascend_passives(
                        &town_context.token.get(),
                        &AscendPassivesRequest {
                            character_id: town_context.character.read().character_id.clone(),
                            passives_tree_state: passives_tree_state.get(),
                        },
                    )
                    .await;
            }
        });
        open.set(false);
    });

    let try_ascend = move |_| {
        (confirm_context.confirm)(
            format! {"Do you confirm Ascension for {} Power Shards?",ascension_cost.get() },
            ascend.clone(),
        );
    };

    let disabled = Signal::derive(move || ascension_cost.get() == 0.0);

    view! {
        <MenuButton on:click=try_ascend disabled=disabled>
            "Confirm Ascension"
        </MenuButton>
    }
}

#[component]
fn ResetButton() -> impl IntoView {
    let confirm_context = expect_context::<ConfirmContext>();

    let reset = Arc::new(move || {});

    let try_reset = move |_| {
        (confirm_context.confirm)("Fully Respec Ascension?".to_string(), reset.clone());
    };

    view! { <MenuButton on:click=try_reset>"Respec Ascension"</MenuButton> }
}

#[component]
fn PassiveSkillTree(
    passives_tree_state: RwSignal<PassivesTreeState>,
    ascension_cost: RwSignal<f64>,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let points_available = Memo::new(move |_| {
        (town_context.character.read().resource_shards - ascension_cost.get()).round()
    });

    let nodes_specs = Arc::new(
        town_context
            .passives_tree_specs
            .read_untracked()
            .nodes
            .clone(),
    );

    // Fake amount of connections to have neatly rendered skill tree
    let amount_connections = Memo::new(|_| 0);

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
                    amount_connections=amount_connections
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
                    passives_tree_state
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
    passives_tree_state: RwSignal<PassivesTreeState>,
) -> impl IntoView {
    let node_level = Memo::new({
        let node_id = node_id.clone();

        move |_| {
            passives_tree_state
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

            let purchase_status = if points_available.get() > 0.0
                && (upgradable || (node_specs.locked && node_level.get() == 0))
            {
                PurchaseStatus::Purchaseable
            } else {
                PurchaseStatus::Inactive
            };

            let meta_status = if node_level.get() > 0 {
                if node_specs.locked && node_level.get() == 1 {
                    MetaStatus::Normal
                } else {
                    MetaStatus::Ascended
                }
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
            passives_tree_state.update(|passives_tree_state| {
                let entry = passives_tree_state
                    .ascended_nodes
                    .entry(node_id.clone())
                    .or_default();
                *entry = entry.saturating_add(1);
            });
            ascension_cost.update(|ascension_cost| *ascension_cost += 1.0); // TODO: Ascend cost?
        }
    };

    view! { <Node node_specs node_status node_level on_click=purchase show_upgrade=true /> }
}
