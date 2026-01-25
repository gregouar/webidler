use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use frontend::components::{
    shared::passives::{
        Connection, MetaStatus, Node, NodeStatus, NodeTooltipContent, PurchaseStatus,
    },
    ui::{
        buttons::MenuButton,
        card::{Card, CardHeader, CardInset, CardTitle},
        dropdown::DropdownMenu,
        input::ValidatedInput,
        pannable::Pannable,
        tooltip::DynamicTooltip,
    },
};
use leptos::{html::*, prelude::*};
use serde::Serialize;
use shared::data::passive::{
    PassiveConnection, PassiveNodeId, PassiveNodeSpecs, PassiveNodeType, PassivesTreeSpecs,
};
use strum::IntoEnumIterator;

use crate::{
    header::HeaderMenu,
    utils::{
        file_loader::{save_json, use_json_loader},
        json_editor::JsonEditor,
    },
};

#[derive(Serialize)]
pub struct SerPassivesTreeSpecs {
    pub nodes: BTreeMap<PassiveNodeId, PassiveNodeSpecs>,
    pub connections: Vec<PassiveConnection>,
}

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
                                    </MenuButton>
                                    <MenuButton on:click=move |_| {
                                        save_json(
                                            &HashMap::from([
                                                (
                                                    "default",
                                                    SerPassivesTreeSpecs {
                                                        nodes: passives_tree_specs
                                                            .read_untracked()
                                                            .nodes
                                                            .clone()
                                                            .into_iter()
                                                            .collect(),
                                                        connections: passives_tree_specs
                                                            .read_untracked()
                                                            .connections
                                                            .clone(),
                                                    },
                                                ),
                                            ]),
                                            "passives.json",
                                        );
                                    }>"Save"</MenuButton>
                                </div>
                            </div>
                            <CardInset pad=false class:flex-1>
                                <PassiveSkillTree passives_tree_specs selected_node />
                            </CardInset>
                        </Card>
                    </div>

                    <Card class="h-full w-xl">
                        <EditNodeMenu passives_tree_specs selected_node />
                    </Card>

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
            <rect x="-5000" y="-5000" width="10000" height="10000" fill="url(#grid)" />
            <For
                each=move || { passives_tree_specs.read().connections.clone().into_iter() }
                key=|conn| (conn.from.clone(), conn.to.clone())
                let(conn)
            >
                <ToolConnection connection=conn passives_tree_specs />
            </For>
            <For
                each=move || {
                    passives_tree_specs.read().nodes.keys().cloned().collect::<Vec<_>>()
                }
                key=|id| id.clone()
                let(id)
            >
                {move || {
                    view! {
                        <ToolNode
                            node_id=id.clone()
                            node_specs=passives_tree_specs
                                .read()
                                .nodes
                                .get(&id)
                                .cloned()
                                .unwrap_or_default()
                            selected_node
                        />
                    }
                }}
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
        {move || {
            selected_node
                .get()
                .map(|node_id| {
                    let node_specs = RwSignal::new(
                        passives_tree_specs.read().nodes.get(&node_id).cloned().unwrap_or_default(),
                    );
                    let on_save = {
                        let node_id = node_id.clone();
                        move |_| {
                            passives_tree_specs
                                .write()
                                .nodes
                                .insert(node_id.clone(), node_specs.get_untracked());
                        }
                    };
                    view! {
                        <CardHeader title="Edit Node" on_close=move || selected_node.set(None)>
                            <div class="flex-1" />
                            <MenuButton class:mr-2 on:click=on_save>
                                "Save"
                            </MenuButton>
                        </CardHeader>
                        <EditNode node_id node_specs />
                    }
                })
        }}
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

    let node_icon = RwSignal::new(Some(node_specs.read_untracked().icon.clone()));
    Effect::new(move || {
        if let Some(node_icon) = node_icon.get() {
            node_specs.write().icon = node_icon;
        }
    });

    let node_x = RwSignal::new(Some(node_specs.read_untracked().x));
    Effect::new(move || {
        if let Some(node_x) = node_x.get() {
            node_specs.write().x = node_x;
        }
    });

    let node_y = RwSignal::new(Some(node_specs.read_untracked().y));
    Effect::new(move || {
        if let Some(node_y) = node_y.get() {
            node_specs.write().y = node_y;
        }
    });

    let node_size = RwSignal::new(Some(node_specs.read_untracked().size));
    Effect::new(move || {
        if let Some(node_size) = node_size.get() {
            node_specs.write().size = node_size;
        }
    });

    let node_type = RwSignal::new(node_specs.read_untracked().node_type.clone());
    Effect::new(move || {
        node_specs.write().node_type = node_type.get();
    });

    let initial_node = RwSignal::new(node_specs.read_untracked().initial_node.clone());
    Effect::new(move || {
        node_specs.write().initial_node = initial_node.get();
    });

    let node_locked = RwSignal::new(node_specs.read_untracked().locked.clone());
    Effect::new(move || {
        node_specs.write().locked = node_locked.get();
    });

    let node_max_level = RwSignal::new(Some(node_specs.read_untracked().max_upgrade_level));
    Effect::new(move || {
        if let Some(node_max_level) = node_max_level.get() {
            node_specs.write().max_upgrade_level = node_max_level;
        }
    });

    let node_effects = RwSignal::new(node_specs.read_untracked().effects.clone());
    Effect::new(move || {
        node_specs.write().effects = node_effects.get();
    });

    let node_triggers = RwSignal::new(node_specs.read_untracked().triggers.clone());
    Effect::new(move || {
        node_specs.write().triggers = node_triggers.get();
    });

    view! {
        <CardInset class="flex-1">
            <div class="text-amber-300">{node_id}</div>
            <ValidatedInput label="Name" id="node_name" input_type="text" bind=node_name />
            <ValidatedInput label="Icon" id="node_icon" input_type="text" bind=node_icon />
            <div class="flex justify-between gap-2">
                <ValidatedInput label="Pos. x" id="x" input_type="number" step="0.5" bind=node_x />
                <ValidatedInput label="Pos. y" id="y" input_type="number" step="0.5" bind=node_y />
            </div>
            <div class="flex justify-between gap-2 items-end">
                <ValidatedInput label="Size" id="size" input_type="number" bind=node_size />
                <DropdownMenu
                    options=PassiveNodeType::iter()
                        .map(|category| (category, serde_plain::to_string(&category).unwrap()))
                        .collect()
                    chosen_option=node_type
                />
            </div>
            <div class="flex justify-between gap-2">
                <div class="flex items-start mt-4">
                    <input
                        id="initial_node"
                        type="checkbox"
                        class="mt-1 mr-2"
                        prop:checked=initial_node
                        on:input=move |ev| initial_node.set(event_target_checked(&ev))
                    />
                    <label for="terms" class="text-sm text-gray-400">
                        "Root Node"
                    </label>
                </div>

                <div class="flex items-start mt-4">
                    <input
                        id="node_locked"
                        type="checkbox"
                        class="mt-1 mr-2"
                        prop:checked=node_locked
                        on:input=move |ev| node_locked.set(event_target_checked(&ev))
                    />
                    <label for="terms" class="text-sm text-gray-400">
                        "Locked"
                    </label>
                </div>
            </div>
            <ValidatedInput
                label="Max Level"
                id="max_level"
                input_type="number"
                bind=node_max_level
            />
            <JsonEditor label="Effects" value=node_effects />
            <JsonEditor label="Triggers" value=node_triggers />
        </CardInset>

        <div>"Result:"</div>
        <CardInset class="space-y-1">
            {move || {
                let node_specs = Arc::new(node_specs.get());
                view! { <NodeTooltipContent node_specs node_level show_upgrade=false /> }
            }}
        </CardInset>
    }
}
