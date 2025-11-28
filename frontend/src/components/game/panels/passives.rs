use leptos::{html::*, prelude::*};

use std::collections::HashMap;
use std::sync::Arc;

use shared::{
    data::passive::{PassiveConnection, PassiveNodeId, PassiveNodeSpecs},
    messages::client::PurchasePassiveMessage,
};

use crate::components::{
    game::game_context::GameContext,
    shared::passives::{
        node_meta_status, Connection, MetaStatus, Node, NodeStatus, PurchaseStatus,
    },
    ui::{
        buttons::CloseButton,
        menu_panel::{MenuPanel, PanelTitle},
        pannable::Pannable,
    },
    websocket::WebsocketContext,
};

#[component]
pub fn PassivesPanel(open: RwSignal<bool>) -> impl IntoView {
    view! {
        <MenuPanel open=open>
            <div class="w-full h-full">
                <div class="bg-zinc-800 rounded-md p-1 xl:p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col gap-1 xl:gap-2 max-h-full">
                    <div class="px-2 xl:px-4 flex items-center justify-between">
                        <PanelTitle>"Passive Skills"</PanelTitle>
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
    let game_context = expect_context::<GameContext>();

    let points_available =
        Memo::new(move |_| game_context.player_resources.read().passive_points > 0);

    let nodes_specs = Arc::new(
        game_context
            .passives_tree_specs
            .read_untracked()
            .nodes
            .clone(),
    );

    view! {
        <Pannable>
            <For
                each=move || {
                    game_context.passives_tree_specs.read().connections.clone().into_iter()
                }
                key=|conn| (conn.from.clone(), conn.to.clone())
                let(conn)
            >
                <InGameConnection connection=conn nodes_specs=nodes_specs.clone() />
            </For>
            <For
                each=move || { game_context.passives_tree_specs.read().nodes.clone().into_iter() }
                key=|(id, _)| id.clone()
                let((id, node))
            >
                <InGameNode node_id=id node_specs=node points_available=points_available />
            </For>
        </Pannable>
    }
}

#[component]
fn InGameNode(
    node_id: PassiveNodeId,
    node_specs: PassiveNodeSpecs,
    points_available: Memo<bool>,
) -> impl IntoView {
    let node_level = Memo::new({
        let game_context = expect_context::<GameContext>();
        let node_id = node_id.clone();

        move |_| {
            game_context
                .passives_tree_state
                .read()
                .ascension
                .ascended_nodes
                .get(&node_id)
                .cloned()
                .unwrap_or_default()
        }
    });

    let node_status = Memo::new({
        let game_context = expect_context::<GameContext>();
        let node_id = node_id.clone();

        move |_| {
            let meta_status = node_meta_status(
                node_level.get(),
                node_specs.locked,
                node_specs.max_upgrade_level,
            );

            let purchase_status = if game_context
                .passives_tree_state
                .read()
                .purchased_nodes
                .contains(&node_id)
            {
                PurchaseStatus::Purchased
            } else if meta_status != MetaStatus::Locked
                && points_available.get()
                && (node_specs.initial_node
                    || game_context
                        .passives_tree_specs
                        .read()
                        .connections
                        .iter()
                        .filter(|connection| {
                            game_context
                                .passives_tree_state
                                .read()
                                .purchased_nodes
                                .contains(&connection.from)
                                || game_context
                                    .passives_tree_state
                                    .read()
                                    .purchased_nodes
                                    .contains(&connection.to)
                        })
                        .any(|connection| connection.from == node_id || connection.to == node_id))
            {
                PurchaseStatus::Purchaseable
            } else {
                PurchaseStatus::Inactive
            };

            NodeStatus {
                purchase_status,
                meta_status,
            }
        }
    });

    let purchase = {
        let conn = expect_context::<WebsocketContext>();
        let game_context = expect_context::<GameContext>();
        move || {
            game_context.player_resources.write().passive_points -= 1;
            game_context
                .passives_tree_state
                .write()
                .purchased_nodes
                .insert(node_id.clone());
            conn.send(
                &PurchasePassiveMessage {
                    node_id: node_id.clone(),
                }
                .into(),
            );
        }
    };

    view! {
        <Node
            node_specs
            node_status
            node_level
            on_click=purchase
            on_right_click=|| {}
            show_upgrade=false
        />
    }
}

#[component]
fn InGameConnection(
    connection: PassiveConnection,
    nodes_specs: Arc<HashMap<String, PassiveNodeSpecs>>,
) -> impl IntoView {
    let amount_connections = Memo::new({
        let game_context = expect_context::<GameContext>();
        let connection_from = connection.from.clone();
        let connection_to = connection.to.clone();

        move |_| {
            game_context
                .passives_tree_state
                .read()
                .purchased_nodes
                .contains(&connection_from) as usize
                + game_context
                    .passives_tree_state
                    .read()
                    .purchased_nodes
                    .contains(&connection_to) as usize
        }
    });

    let node_levels = Memo::new({
        let game_context = expect_context::<GameContext>();
        let connection_from = connection.from.clone();
        let connection_to = connection.to.clone();

        move |_| {
            (
                game_context
                    .passives_tree_state
                    .read()
                    .ascension
                    .ascended_nodes
                    .get(&connection_from)
                    .cloned()
                    .unwrap_or_default(),
                game_context
                    .passives_tree_state
                    .read()
                    .ascension
                    .ascended_nodes
                    .get(&connection_to)
                    .cloned()
                    .unwrap_or_default(),
            )
        }
    });

    view! { <Connection connection nodes_specs amount_connections node_levels /> }
}
