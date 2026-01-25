use std::{collections::HashMap, sync::Arc};

use frontend::components::{
    shared::passives::{Connection, MetaStatus, Node, NodeStatus, PurchaseStatus},
    ui::{
        buttons::MenuButton,
        card::{Card, CardInset, CardTitle},
        pannable::Pannable,
        tooltip::DynamicTooltip,
    },
};
use leptos::{html::*, leptos_dom::logging::console_log, prelude::*};
use shared::data::passive::{
    PassiveConnection, PassiveNodeId, PassiveNodeSpecs, PassivesTreeSpecs,
};

use crate::{header::HeaderMenu, utils::file_loader::use_json_loader};

#[component]
pub fn PassivesPage() -> impl IntoView {
    let (loaded_file, on_skills_file) = use_json_loader::<HashMap<String, PassivesTreeSpecs>>();
    let passives_tree_specs = RwSignal::new(Default::default());

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
                <div class="absolute inset-0 flex flex-col p-1 xl:p-4 items-center">
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
                            <CardInset pad=false>
                                <PassiveSkillTree passives_tree_specs />
                            </CardInset>
                        </Card>
                    </div>
                </div>
            </div>
        </main>
    }
}

#[component]
fn PassiveSkillTree(passives_tree_specs: RwSignal<PassivesTreeSpecs>) -> impl IntoView {
    // THIS FEELS WRONG
    let nodes_specs = Arc::new(passives_tree_specs.read_untracked().nodes.clone());

    view! {
        <Pannable>
            <For
                each=move || { passives_tree_specs.read().connections.clone().into_iter() }
                key=|conn| (conn.from.clone(), conn.to.clone())
                let(conn)
            >
                <ToolConnection connection=conn nodes_specs=nodes_specs.clone() />
            </For>
            <For
                each=move || { passives_tree_specs.read().nodes.clone().into_iter() }
                key=|(id, _)| id.clone()
                let((id, node))
            >
                <ToolNode node_id=id node_specs=node />
            </For>
        </Pannable>
    }
}

#[component]
fn ToolNode(node_id: PassiveNodeId, node_specs: PassiveNodeSpecs) -> impl IntoView {
    let node_status = Memo::new(|_| NodeStatus {
        purchase_status: PurchaseStatus::Purchased,
        meta_status: MetaStatus::Normal,
    });
    let node_level = Memo::new(|_| 1);

    view! {
        <Node
            node_specs
            node_status
            node_level
            on_click=move || {}
            on_right_click=move || {}
            show_upgrade=true
        />
    }
}

#[component]
fn ToolConnection(
    connection: PassiveConnection,
    nodes_specs: Arc<HashMap<String, PassiveNodeSpecs>>,
) -> impl IntoView {
    let amount_connections = Memo::new(|_| 2);
    let node_levels = Memo::new(|_| (1, 1));

    view! { <Connection connection nodes_specs amount_connections node_levels /> }
}
