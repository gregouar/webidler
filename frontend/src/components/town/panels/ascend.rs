use leptos::{html::*, prelude::*};

use std::sync::Arc;

use shared::data::passive::{PassiveNodeId, PassiveNodeSpecs};

use crate::components::{
    game::panels::passives::{Connection, MetaStatus, Node, NodeStatus, PurchaseStatus},
    town::TownContext,
    ui::{buttons::CloseButton, menu_panel::MenuPanel, pannable::Pannable},
};

#[component]
pub fn AscendPanel(open: RwSignal<bool>) -> impl IntoView {
    view! {
        <MenuPanel open=open>
            <div class="w-full p-4">
                <div class="bg-zinc-800 rounded-md p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col gap-2">
                    <div class="px-4 relative z-10 flex items-center justify-between">
                        <span class="text-shadow-md shadow-gray-950 text-amber-200 text-xl font-semibold">
                            "Passive Skills "
                        </span>
                        <CloseButton on:click=move |_| open.set(false) />
                    </div>

                    <PassiveSkillTree />
                </div>
            </div>
        </MenuPanel>
    }
}

#[component]
fn PassiveSkillTree() -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    // let points_available =
    //     Memo::new(move |_| game_context.player_resources.read().passive_points > 0);

    let nodes_specs = Arc::new(
        town_context
            .passives_tree_specs
            .read_untracked()
            .nodes
            .clone(),
    );

    // Fake amount of connections to have neatly rendered skill tree
    let amount_connections = Memo::new(|_| 1);

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
                <AscendNode node_id=id node_specs=node />
            </For>
        </Pannable>
    }
}

#[component]
fn AscendNode(node_id: PassiveNodeId, node_specs: PassiveNodeSpecs) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let node_status = Memo::new({
        let node_id = node_id.clone();
        move |_| {
            let purchase_status = PurchaseStatus::Purchaseable;

            let ascend_level = town_context
                .passives_tree_state
                .read()
                .ascended_nodes
                .get(&node_id)
                .cloned()
                .unwrap_or_default();

            let meta_status = if ascend_level > 0 {
                if node_specs.locked && ascend_level == 1 {
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
            town_context
                .passives_tree_state
                .update(|passives_tree_state| {
                    let entry = passives_tree_state
                        .ascended_nodes
                        .entry(node_id.clone())
                        .or_default();
                    *entry = entry.saturating_add(1);
                });
        }
    };

    view! { <Node node_specs node_status on_click=purchase /> }
}
