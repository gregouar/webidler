use frontend::components::ui::{
    buttons::MenuButton,
    card::{Card, CardInset, CardTitle},
    pannable::Pannable,
};
use leptos::{html::*, prelude::*};

use crate::header::HeaderMenu;

#[component]
pub fn PassivesPage() -> impl IntoView {

    let passives_tree = ;

    view! {
        <main class="my-0 mx-auto w-full text-center overflow-x-hidden flex flex-col min-h-screen">
            <HeaderMenu />
            <div class="relative flex-1">
                <Card>
                    <div class="flex flex-between mx-4">
                        <CardTitle>"Passives"</CardTitle>
                        <MenuButton>"Save"</MenuButton>
                    </div>
                    <CardInset pad=false>
                        <div class="w-full h-full">
                            <PassiveSkillTree />
                        </div>
                    </CardInset>
                </Card>
            </div>
        </main>
    }
}

#[component]
fn PassiveSkillTree() -> impl IntoView {
    view! {
        <Pannable>
            <For
                each=move || {
                    town_context.passives_tree_specs.read().connections.clone().into_iter()
                }
                key=|conn| (conn.from.clone(), conn.to.clone())
                let(conn)
            >
                <ToolConnection
                    connection=conn
                    nodes_specs=nodes_specs.clone()
                    passives_tree_ascension
                />
            </For>
            <For
                each=move || { town_context.passives_tree_specs.read().nodes.clone().into_iter() }
                key=|(id, _)| id.clone()
                let((id, node))
            >
                <ToolNode
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
fn ToolNode(
    node_id: PassiveNodeId,
    node_specs: PassiveNodeSpecs,
    points_available: Memo<f64>,
    ascension_cost: RwSignal<f64>,
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
    view_only: bool,
) -> impl IntoView {
    let town_context: TownContext = expect_context();

    let node_level = Memo::new({
        let node_id = node_id.clone();

        move |_| {
            passives_tree_ascension
                .read()
                .ascended_nodes
                .get(&node_id)
                .copied()
                .unwrap_or_default()
        }
    });

    let max_node_level = if node_specs.upgrade_effects.is_empty() {
        node_specs.locked as u8
    } else {
        node_specs.max_upgrade_level.unwrap_or(u8::MAX)
    };

    let max_upgrade_level = Memo::new({
        let node_id = node_id.clone();
        move |_| {
            let max_connection_level = if node_specs.initial_node {
                u8::MAX
            } else {
                town_context
                    .passives_tree_specs
                    .read()
                    .connections
                    .iter()
                    .filter_map(|connection| {
                        if connection.from == node_id {
                            Some(connection.to.clone())
                        } else if connection.to == node_id {
                            Some(connection.from.clone())
                        } else {
                            None
                        }
                    })
                    .map(|node_id| {
                        passives_tree_ascension
                            .read()
                            .ascended_nodes
                            .get(&node_id)
                            .copied()
                            .unwrap_or_default()
                    })
                    .max()
                    .unwrap_or_default()
            };

            max_node_level.min(max_connection_level)
        }
    });

    let node_status = Memo::new({
        move |_| {
            let upgradable = max_upgrade_level.get() > node_level.get();
            // let maxed = node_level.get() >= max_upgrade_level && node_level.get() > 0;

            let purchase_status =
            //  if maxed {
            //     PurchaseStatus::Inactive
            // } else 
            if (view_only|| node_level.get() == max_node_level) && node_level.get() > 0  {
                PurchaseStatus::Purchased
            } else if points_available.get() > 0.0 && upgradable
                // && (upgradable || (node_specs.locked && node_level.get() == 0))
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

#[component]
fn ToolConnection(
    connection: PassiveConnection,
    nodes_specs: Arc<HashMap<String, PassiveNodeSpecs>>,
    passives_tree_ascension: RwSignal<PassivesTreeAscension>,
) -> impl IntoView {
    let amount_connections = Memo::new({
        let connection_from = connection.from.clone();
        let connection_to = connection.to.clone();

        move |_| {
            passives_tree_ascension
                .read()
                .ascended_nodes
                .contains_key(&connection_from) as usize
                + passives_tree_ascension
                    .read()
                    .ascended_nodes
                    .contains_key(&connection_to) as usize
        }
    });

    let node_levels = Memo::new({
        let connection_from = connection.from.clone();
        let connection_to = connection.to.clone();

        move |_| {
            (
                passives_tree_ascension
                    .read()
                    .ascended_nodes
                    .get(&connection_from)
                    .cloned()
                    .unwrap_or_default(),
                passives_tree_ascension
                    .read()
                    .ascended_nodes
                    .get(&connection_to)
                    .cloned()
                    .unwrap_or_default(),
            )
        }
    });

    view! { <Connection connection nodes_specs amount_connections node_levels /> }
}
