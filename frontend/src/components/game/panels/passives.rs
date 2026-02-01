use leptos::{html::*, prelude::*};

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
        card::{Card, CardHeader, CardInset},
        menu_panel::MenuPanel,
        pannable::Pannable,
    },
    websocket::WebsocketContext,
};

#[component]
pub fn PassivesPanel(open: RwSignal<bool>) -> impl IntoView {
    view! {
        <MenuPanel open=open>
            <div class="w-full h-full">
                <Card>
                    <CardHeader title="Passive Skills" on_close=move || open.set(false) />
                    <CardInset pad=false>
                        <PassiveSkillTree />
                    </CardInset>
                </Card>
            </div>
        </MenuPanel>
    }
}

#[component]
fn PassiveSkillTree() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let points_available =
        Memo::new(move |_| game_context.player_resources.read().passive_points > 0);

    view! {
        <Pannable>
            <For
                each=move || {
                    game_context.passives_tree_specs.read().connections.clone().into_iter()
                }
                key=|conn| (conn.from.clone(), conn.to.clone())
                let(conn)
            >
                <InGameConnection connection=conn />
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
    let game_context: GameContext = expect_context();
    let node_level = Memo::new({
        let node_id = node_id.clone();

        move |_| {
            game_context
                .passives_tree_state
                .read()
                .ascension
                .ascended_nodes
                .get(&node_id)
                .copied()
                .unwrap_or_default()
        }
    });

    let node_status = Memo::new({
        let node_id = node_id.clone();

        move |_| {
            let meta_status = node_meta_status(
                node_level.get(),
                node_specs.locked,
                // node_specs.max_upgrade_level,
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
fn InGameConnection(connection: PassiveConnection) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let amount_connections = Memo::new({
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

    view! {
        <Connection
            connection
            passives_tree_specs=game_context.passives_tree_specs
            amount_connections
            node_levels
        />
    }
}
