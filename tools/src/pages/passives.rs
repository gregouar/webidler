use std::{collections::HashMap, sync::Arc};

use frontend::components::{
    shared::passives::{
        Connection, MetaStatus, Node, NodeStatus, NodeTooltipContent, PurchaseStatus,
    },
    ui::{
        buttons::MenuButton,
        card::{Card, CardHeader, CardInset, CardTitle},
        input::ValidatedInput,
        pannable::Pannable,
        tooltip::DynamicTooltip,
    },
};
use leptos::{html::*, prelude::*};
use shared::data::passive::{
    PassiveConnection, PassiveNodeId, PassiveNodeSpecs, PassivesTreeSpecs,
};

use crate::{header::HeaderMenu, utils::file_loader::use_json_loader};

#[component]
pub fn PassivesPage() -> impl IntoView {
    let (loaded_file, on_skills_file) = use_json_loader::<HashMap<String, PassivesTreeSpecs>>();
    let passives_tree_specs = RwSignal::new(Default::default());

    let selected_node: RwSignal<Option<PassiveNodeId>> = RwSignal::new(None);

    Effect::new(move || {
        loaded_file.with(|loaded_file| {
            if let Some(specs) = loaded_file.as_ref().and_then(|f| f.get("default")) {
                passives_tree_specs.set(specs.clone());
            }
        });
    });

    view! {
        <main class="my-0 mx-auto w-full text-center overflow-x-hidden flex flex-col min-h-screen">
            <DynamicTooltip />
            <HeaderMenu />
            <div class="relative flex-1">
                <div class="absolute inset-0 flex p-1 xl:p-4 items-center gap-4">
                    <div class="w-full h-full">
                        <Card>
                            <div class="flex justify-between mx-4 items-center">
                                <CardTitle>"Passives"</CardTitle>

                                <div class="flex gap-2">
                                    <MenuButton>
                                        <input type="file" on:change=on_skills_file />
                                    // "Load"
                                    </MenuButton>
                                    <MenuButton>"Save"</MenuButton>
                                </div>
                            </div>
                            <CardInset pad=false class:flex-1>
                                <PassiveSkillTree passives_tree_specs selected_node />
                            </CardInset>
                        </Card>
                    </div>

                    <EditNodeMenu passives_tree_specs selected_node />

                </div>
            </div>
        </main>
    }
}

#[component]
fn PassiveSkillTree(
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    selected_node: RwSignal<Option<PassiveNodeId>>,
) -> impl IntoView {
    view! {
        <Pannable>
            <For
                each=move || { passives_tree_specs.read().connections.clone().into_iter() }
                key=|conn| (conn.from.clone(), conn.to.clone())
                let(conn)
            >
                <ToolConnection connection=conn passives_tree_specs />
            </For>
            <For
                each=move || { passives_tree_specs.read().nodes.clone().into_iter() }
                key=|(id, _)| id.clone()
                let((id, node))
            >
                <ToolNode node_id=id node_specs=node selected_node />
            </For>
        </Pannable>
    }
}

#[component]
fn ToolNode(
    node_id: PassiveNodeId,
    node_specs: PassiveNodeSpecs,
    selected_node: RwSignal<Option<PassiveNodeId>>,
) -> impl IntoView {
    let node_status = Memo::new(move |_| NodeStatus {
        purchase_status: PurchaseStatus::Purchaseable,
        meta_status: match node_specs.locked {
            true => MetaStatus::Locked,
            false => MetaStatus::Normal,
        },
    });
    let node_level = Memo::new(|_| 0);

    view! {
        <Node
            node_specs
            node_status
            node_level
            on_click=move || { selected_node.set(Some(node_id.clone())) }
            on_right_click=move || {}
            show_upgrade=true
        />
    }
}

#[component]
fn ToolConnection(
    connection: PassiveConnection,
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
) -> impl IntoView {
    let amount_connections = Memo::new(|_| 1);
    let node_levels = Memo::new(|_| (0, 0));

    view! { <Connection connection passives_tree_specs amount_connections node_levels /> }
}

#[component]
fn EditNodeMenu(
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    selected_node: RwSignal<Option<PassiveNodeId>>,
) -> impl IntoView {
    view! {
        <Card class="h-full w-xl">
            <CardHeader title="Edit Node" on_close=move || selected_node.set(None)>
                <div class="flex-1" />
                <MenuButton class:mr-2>"Save"</MenuButton>
            </CardHeader>
            {move || match selected_node.get() {
                Some(selected_node) => {
                    let node_specs = RwSignal::new(
                        passives_tree_specs
                            .read_untracked()
                            .nodes
                            .get(&selected_node)
                            .cloned()
                            .unwrap_or_default(),
                    );

                    view! { <EditNode node_id=selected_node node_specs /> }
                        .into_any()
                }
                None => view! {}.into_any(),
            }}
        </Card>
    }
}

#[component]
fn EditNode(node_id: PassiveNodeId, node_specs: RwSignal<PassiveNodeSpecs>) -> impl IntoView {
    let node_level = Memo::new(|_| 0);

    let node_name = RwSignal::new(Some(node_specs.read_untracked().name.clone()));
    Effect::new(move || {
        if let Some(node_name) = node_name.get() {
            node_specs.write().name = node_name;
        }
    });

    view! {
        <CardInset class="flex-1 space-y-2">
            <div class="text-amber-300">{node_id}</div>
            <ValidatedInput
                label="Name"
                id="node_name"
                input_type="text"
                placeholder="Node Name"
                bind=node_name
            />
        </CardInset>
        <div>"Result:"</div>
        <CardInset class="space-y-2">
            {move || {
                let node_specs = Arc::new(node_specs.get());
                view! { <NodeTooltipContent node_specs node_level show_upgrade=false /> }
            }}
        </CardInset>
    }
}
