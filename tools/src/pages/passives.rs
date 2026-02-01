use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

use frontend::components::{
    events::{EventsContext, Key},
    shared::passives::{
        Connection, MetaStatus, Node, NodeStatus, NodeTooltipContent, PurchaseStatus,
    },
    ui::{
        buttons::{MenuButton, TabButton},
        card::{Card, CardHeader, CardInset, CardTitle},
        confirm::ConfirmContext,
        dropdown::DropdownMenu,
        input::ValidatedInput,
        pannable::Pannable,
        tooltip::DynamicTooltip,
    },
};
use leptos::{html::*, prelude::*};
use leptos_use::{WatchDebouncedOptions, watch_debounced_with_options};
use serde::Serialize;
use shared::data::passive::{
    PassiveConnection, PassiveNodeId, PassiveNodeSpecs, PassiveNodeType, PassivesTreeSpecs,
};
use strum::IntoEnumIterator;

use crate::{
    header::HeaderMenu,
    utils::{
        file_loader::{save_json, use_json_loader},
        history_tracker::HistoryTracker,
        json_editor::JsonEditor,
    },
};

#[derive(Serialize)]
struct SerPassivesTreeSpecs {
    nodes: BTreeMap<PassiveNodeId, PassiveNodeSpecs>,
    connections: Vec<PassiveConnection>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum ToolMode {
    Add,
    Edit,
    Connect,
}

#[component]
pub fn PassivesPage() -> impl IntoView {
    let events_context: EventsContext = expect_context();

    let (loaded_file, on_skills_file) = use_json_loader::<HashMap<String, PassivesTreeSpecs>>();
    let passives_tree_specs = RwSignal::new(Default::default());
    let passives_history_tracker = RwSignal::new(HistoryTracker::<PassivesTreeSpecs>::new(
        100,
        passives_tree_specs.get_untracked(),
    ));

    let selected_node: RwSignal<Option<PassiveNodeId>> = RwSignal::new(None);
    let clipboard_node = RwSignal::new(None);
    let tool_mode = RwSignal::new(ToolMode::Edit);
    let clicked_tool_mode = RwSignal::new(ToolMode::Edit);

    Effect::new(move || {
        loaded_file.with(|loaded_file| {
            if let Some(specs) = loaded_file.as_ref().and_then(|f| f.get("default")) {
                passives_tree_specs.set(specs.clone());
                record_history(passives_history_tracker, passives_tree_specs);
            }
        });
    });

    let save = move || {
        save_json(
            &HashMap::from([(
                "default",
                SerPassivesTreeSpecs {
                    nodes: passives_tree_specs
                        .read_untracked()
                        .nodes
                        .clone()
                        .into_iter()
                        .collect(),
                    connections: passives_tree_specs.read_untracked().connections.clone(),
                },
            )]),
            "passives.json",
        );
    };

    let file_input: NodeRef<Input> = NodeRef::new();

    let load = move || {
        if let Some(input) = file_input.get() {
            input.click();
        }
    };

    Effect::new({
        move || {
            if events_context.key_pressed(Key::Ctrl) {
                if events_context.key_pressed(Key::Character('z')) {
                    undo_history(passives_history_tracker, passives_tree_specs);
                }
                if events_context.key_pressed(Key::Character('y')) {
                    redo_history(passives_history_tracker, passives_tree_specs);
                }
                if events_context.key_pressed(Key::Character('s')) {
                    save();
                } else if events_context.key_pressed(Key::Character('o')) {
                    load();
                }
            } else if events_context.key_pressed(Key::Alt) {
                tool_mode.set(ToolMode::Add);
            } else if events_context.key_pressed(Key::Shift) {
                tool_mode.set(ToolMode::Connect);
            } else {
                tool_mode.set(clicked_tool_mode.get_untracked());
            }
        }
    });

    view! {
        <main class="my-0 mx-auto w-full text-center overflow-x-hidden flex flex-col min-h-screen">
            <DynamicTooltip />
            <input
                node_ref=file_input
                type="file"
                accept="application/json"
                on:change=on_skills_file
                class="hidden"
            />
            <HeaderMenu />
            <div class="relative flex-1">
                <div class="absolute inset-0 flex p-1 xl:p-4 items-center gap-4">
                    <div class="w-full h-full">
                        <Card>
                            <div class="flex justify-between mx-4 items-center">
                                <CardTitle>"Passives"</CardTitle>

                                <div class="flex gap-2 ml-4">
                                    <MenuButton on:click=move |_| { load() }>"Load"</MenuButton>
                                    <MenuButton on:click=move |_| { save() }>"Save"</MenuButton>
                                </div>

                                <div class="flex-1" />

                                <div class="flex gap-2 -mb-4 items-end">
                                    <TabButton
                                        is_active=Signal::derive(move || {
                                            tool_mode.get() == ToolMode::Add
                                        })
                                        on:click=move |_| {
                                            tool_mode.set(ToolMode::Add);
                                            clicked_tool_mode.set(ToolMode::Add);
                                        }
                                    >

                                        "Add"
                                    </TabButton>
                                    <TabButton
                                        is_active=Signal::derive(move || {
                                            tool_mode.get() == ToolMode::Connect
                                        })
                                        on:click=move |_| {
                                            selected_node.set(None);
                                            tool_mode.set(ToolMode::Connect);
                                            clicked_tool_mode.set(ToolMode::Connect);
                                        }
                                    >
                                        "Connect"
                                    </TabButton>
                                    <TabButton
                                        is_active=Signal::derive(move || {
                                            tool_mode.get() == ToolMode::Edit
                                        })
                                        on:click=move |_| {
                                            tool_mode.set(ToolMode::Edit);
                                            clicked_tool_mode.set(ToolMode::Edit);
                                        }
                                    >
                                        "Edit"
                                    </TabButton>
                                </div>

                                <div class="flex-1" />

                                <div class="flex gap-2 ">
                                    <MenuButton
                                        on:click=move |_| undo_history(
                                            passives_history_tracker,
                                            passives_tree_specs,
                                        )
                                        disabled=Signal::derive(move || {
                                            !passives_history_tracker.read().can_undo()
                                        })
                                    >
                                        "Undo"
                                    </MenuButton>
                                    <MenuButton
                                        on:click=move |_| redo_history(
                                            passives_history_tracker,
                                            passives_tree_specs,
                                        )
                                        disabled=Signal::derive(move || {
                                            !passives_history_tracker.read().can_redo()
                                        })
                                    >
                                        "Redo"
                                    </MenuButton>
                                </div>

                            </div>
                            <CardInset pad=false class:flex-1 class:z-1>
                                <PassiveSkillTree
                                    passives_tree_specs
                                    passives_history_tracker
                                    selected_node
                                    clipboard_node
                                    tool_mode
                                />
                            </CardInset>
                        </Card>
                    </div>

                    <Card class="h-full w-2xl">
                        {move || match tool_mode.get() {
                            ToolMode::Edit => {
                                view! {
                                    <EditNodeMenu
                                        passives_tree_specs
                                        passives_history_tracker
                                        selected_node
                                        clipboard_node
                                    />
                                }
                                    .into_any()
                            }
                            ToolMode::Connect | ToolMode::Add => ().into_any(),
                        }}

                    </Card>

                </div>
            </div>
        </main>
    }
}

#[component]
fn PassiveSkillTree(
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    passives_history_tracker: RwSignal<HistoryTracker<PassivesTreeSpecs>>,
    selected_node: RwSignal<Option<PassiveNodeId>>,
    clipboard_node: RwSignal<Option<PassiveNodeSpecs>>,
    tool_mode: RwSignal<ToolMode>,
) -> impl IntoView {
    let events_context: EventsContext = expect_context();
    let mouse_position = RwSignal::new((0.0, 0.0));

    Effect::new({
        let pasted = RwSignal::new(false);
        move || {
            if selected_node.read().is_none()
                && events_context.key_pressed(Key::Ctrl)
                && events_context.key_pressed(Key::Character('v'))
            {
                if !pasted.get_untracked() {
                    pasted.set(true);
                    paste_node(
                        &add_node(
                            passives_tree_specs,
                            passives_history_tracker,
                            mouse_position,
                        ),
                        passives_tree_specs,
                        passives_history_tracker,
                        clipboard_node,
                    );
                }
            } else {
                pasted.set(false);
            }
        }
    });

    view! {
        <Pannable
            mouse_position
            on:click=move |_| handle_click_outside(
                mouse_position,
                passives_tree_specs,
                passives_history_tracker,
                selected_node,
                tool_mode,
            )
            on:contextmenu=move |ev| {
                ev.prevent_default();
            }
        >
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
                            passives_tree_specs
                            passives_history_tracker
                            selected_node
                            tool_mode
                            mouse_position
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
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    passives_history_tracker: RwSignal<HistoryTracker<PassivesTreeSpecs>>,
    selected_node: RwSignal<Option<PassiveNodeId>>,
    tool_mode: RwSignal<ToolMode>,
    mouse_position: RwSignal<(f64, f64)>,
) -> impl IntoView {
    let node_specs = {
        let node_id = node_id.clone();
        move || {
            passives_tree_specs
                .read()
                .nodes
                .get(&node_id)
                .cloned()
                .unwrap_or_default()
        }
    };
    let node_specs_untracked = {
        let node_id = node_id.clone();
        move || {
            passives_tree_specs
                .read_untracked()
                .nodes
                .get(&node_id)
                .cloned()
                .unwrap_or_default()
        }
    };

    let node_status = Memo::new({
        let node_id = node_id.clone();
        let node_specs = node_specs.clone();
        move |_| NodeStatus {
            purchase_status: match selected_node.read() == Some(node_id.clone()) {
                true => PurchaseStatus::Purchased,
                false => PurchaseStatus::Purchaseable,
            },
            meta_status: match (
                node_specs().locked,
                selected_node.read() == Some(node_id.clone()),
            ) {
                (_, true) => MetaStatus::Ascended,
                (true, _) => MetaStatus::Locked,
                _ => MetaStatus::Normal,
            },
        }
    });

    let node_level = Memo::new({
        let node_specs = node_specs.clone();
        move |_| node_specs().max_upgrade_level.unwrap_or_default()
    });

    let on_click = {
        let node_id = node_id.clone();
        move || {
            handle_click_node(
                node_id.clone(),
                passives_tree_specs,
                passives_history_tracker,
                selected_node,
                tool_mode,
            )
        }
    };

    let dragging_start = RwSignal::new(None::<((f64, f64), (f64, f64))>);
    Effect::new({
        let node_id = node_id.clone();
        move |_| {
            if let ToolMode::Edit = tool_mode.get()
                && let Some((mouse_start, node_start)) = dragging_start.get()
            {
                let mouse_position = mouse_position.get();

                let delta = mouse_position_to_node_position((
                    mouse_position.0 - mouse_start.0,
                    mouse_position.1 - mouse_start.1,
                ));

                if let Some(node) = passives_tree_specs.write().nodes.get_mut(&node_id) {
                    (node.x, node.y) = (node_start.0 + delta.0, node_start.1 + delta.1);
                }
            }
        }
    });

    let on_mousedown = {
        let node_id = node_id.clone();
        let node_specs_untracked = node_specs_untracked.clone();
        move |ev: web_sys::MouseEvent| {
            if ev.button() == 0
                && let ToolMode::Edit = tool_mode.get_untracked()
                && dragging_start.get_untracked().is_none()
                && selected_node.get() == Some(node_id.clone())
            {
                let node_specs = node_specs_untracked();
                dragging_start.set(Some((
                    mouse_position.get_untracked(),
                    (node_specs.x, node_specs.y),
                )));
            }
        }
    };

    let on_mouseup = {
        let node_specs_untracked = node_specs_untracked.clone();
        move |ev: web_sys::MouseEvent| {
            if ev.button() == 0
                && let ToolMode::Edit = tool_mode.get_untracked()
            {
                if let Some((_, old_node_pos)) = dragging_start.get_untracked() {
                    let node_specs = node_specs_untracked();
                    if old_node_pos != (node_specs.x, node_specs.y) {
                        record_history(passives_history_tracker, passives_tree_specs);
                    }
                }
                dragging_start.set(None);
            }
        }
    };

    view! {
        {move || {
            view! {
                <Node
                    node_specs=node_specs()
                    node_status
                    node_level
                    on_click=on_click.clone()
                    on_right_click=move || {}
                    show_upgrade=true
                    on:mousedown=on_mousedown.clone()
                    on:mouseup=on_mouseup.clone()
                />
            }
        }}
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
    passives_history_tracker: RwSignal<HistoryTracker<PassivesTreeSpecs>>,
    selected_node: RwSignal<Option<PassiveNodeId>>,
    clipboard_node: RwSignal<Option<PassiveNodeSpecs>>,
) -> impl IntoView {
    let events_context: EventsContext = expect_context();

    let on_copy = move || {
        if let Some(node_id) = selected_node.get_untracked() {
            clipboard_node.set(Some(
                passives_tree_specs
                    .read_untracked()
                    .nodes
                    .get(&node_id)
                    .cloned()
                    .unwrap_or_default(),
            ));
        }
    };
    let on_paste = move || {
        if let Some(node_id) = selected_node.get_untracked() {
            paste_node(
                &node_id,
                passives_tree_specs,
                passives_history_tracker,
                clipboard_node,
            )
        }
    };
    let do_delete_node = Arc::new({
        move || {
            if let Some(node_id) = selected_node.get_untracked() {
                delete_node(&node_id, passives_tree_specs, passives_history_tracker);
                selected_node.set(None);
            }
        }
    });
    Effect::new({
        let pasted = RwSignal::new(false);
        let copied = RwSignal::new(false);
        move || {
            if events_context.key_pressed(Key::Ctrl) {
                if events_context.key_pressed(Key::Character('c')) {
                    if !copied.get_untracked() {
                        on_copy();
                        copied.set(true);
                    }
                } else {
                    copied.set(false);
                }
                if events_context.key_pressed(Key::Character('v')) {
                    if !pasted.get_untracked() {
                        on_paste();
                        pasted.set(true);
                    }
                } else {
                    pasted.set(false);
                }
            }
        }
    });

    view! {
        {move || {
            selected_node
                .get()
                .map(|node_id| {
                    let node_specs = RwSignal::new(
                        passives_tree_specs.read().nodes.get(&node_id).cloned().unwrap_or_default(),
                    );
                    let try_delete_node = {
                        let confirm_context: ConfirmContext = expect_context();
                        let do_delete_node = do_delete_node.clone();
                        move |_| {
                            (confirm_context
                                .confirm)(
                                "Confirm delete node?".to_string(),
                                do_delete_node.clone(),
                            );
                        }
                    };
                    let _ = watch_debounced_with_options(
                        move || node_specs.get(),
                        move |value, _, _| {
                            if let Some(node_id) = selected_node.get_untracked()
                                && passives_tree_specs
                                    .read_untracked()
                                    .nodes
                                    .get(&node_id)
                                    .map(|node_specs| *node_specs != *value)
                                    .unwrap_or_default()
                            {
                                passives_tree_specs
                                    .write()
                                    .nodes
                                    .insert(node_id.clone(), value.clone());
                                record_history(passives_history_tracker, passives_tree_specs);
                            }
                        },
                        500.0,
                        WatchDebouncedOptions::default().immediate(false),
                    );
                    // let on_save = {
                    // move || {
                    // if let Some(node_id) = selected_node.get_untracked() {
                    // passives_tree_specs
                    // .write()
                    // .nodes
                    // .insert(node_id.clone(), node_specs.get_untracked());
                    // record_history(passives_history_tracker, passives_tree_specs);
                    // }
                    // }
                    // };

                    view! {
                        <CardHeader
                            title="Edit Node"
                            on_close=move || selected_node.set(None)
                            class:gap-2
                        >
                            <MenuButton class:ml-2 on:click=try_delete_node>
                                "❌"
                            </MenuButton>
                            <div class="flex-1" />
                            <MenuButton on:click=move |_| on_copy()>"Copy"</MenuButton>
                            <MenuButton class:mr-2 on:click=move |_| on_paste()>
                                "Paste"
                            </MenuButton>
                        // <MenuButton class:mr-2 on:click=move |_| on_save()>
                        // "Save"
                        // </MenuButton>
                        </CardHeader>
                        <EditNode node_id node_specs />
                    }
                })
        }}
    }
}

#[component]
fn EditNode(node_id: PassiveNodeId, node_specs: RwSignal<PassiveNodeSpecs>) -> impl IntoView {
    // TODO: Allow to choose level to test
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

    let node_type = RwSignal::new(node_specs.read_untracked().node_type);
    Effect::new(move || {
        node_specs.write().node_type = node_type.get();
    });

    let initial_node = RwSignal::new(node_specs.read_untracked().initial_node);
    Effect::new(move || {
        node_specs.write().initial_node = initial_node.get();
    });

    let node_locked = RwSignal::new(node_specs.read_untracked().locked);
    Effect::new(move || {
        node_specs.write().locked = node_locked.get();
    });

    let node_socket = RwSignal::new(node_specs.read_untracked().socket);
    Effect::new(move || {
        node_specs.write().socket = node_socket.get();
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

    let node_upgrades = RwSignal::new(node_specs.read_untracked().upgrade_effects.clone());
    Effect::new(move || {
        node_specs.write().upgrade_effects = node_upgrades.get();
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

                <div class="flex items-start mt-4">
                    <input
                        id="node_socket"
                        type="checkbox"
                        class="mt-1 mr-2"
                        prop:checked=node_socket
                        on:input=move |ev| node_socket.set(event_target_checked(&ev))
                    />
                    <label for="terms" class="text-sm text-gray-400">
                        "Socket"
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
            <JsonEditor label="Upgrade Effects" value=node_upgrades />
            <JsonEditor label="Triggers" value=node_triggers />
        </CardInset>

        <div>"Result:"</div>
        <CardInset class="space-y-1">
            {move || {
                let node_specs = node_specs.get();
                view! { <NodeTooltipContent node_specs node_level show_upgrade=true /> }
            }}
        </CardInset>
    }
}

fn mouse_position_to_node_position(mouse_position: (f64, f64)) -> (f64, f64) {
    let (x, y) = mouse_position;
    (
        (x * 0.1 / 2.5).round() * 2.5,
        -(y * 0.1 / 2.5).round() * 2.5,
    )
}

fn handle_click_outside(
    mouse_position: RwSignal<(f64, f64)>,
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    passives_history_tracker: RwSignal<HistoryTracker<PassivesTreeSpecs>>,
    selected_node: RwSignal<Option<PassiveNodeId>>,
    tool_mode: RwSignal<ToolMode>,
) {
    match tool_mode.get_untracked() {
        ToolMode::Edit | ToolMode::Connect => selected_node.set(None),
        ToolMode::Add => {
            selected_node.set(Some(add_node(
                passives_tree_specs,
                passives_history_tracker,
                mouse_position,
            )));
            tool_mode.set(ToolMode::Edit);
        }
    }
}

fn handle_click_node(
    node_id: PassiveNodeId,
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    passives_history_tracker: RwSignal<HistoryTracker<PassivesTreeSpecs>>,
    selected_node: RwSignal<Option<PassiveNodeId>>,
    tool_mode: RwSignal<ToolMode>,
) {
    match tool_mode.get_untracked() {
        ToolMode::Edit => selected_node.set(Some(node_id)),
        ToolMode::Connect => match selected_node.get_untracked() {
            Some(selected_node_id) if selected_node_id == node_id => selected_node.set(None),
            Some(selected_node_id) => {
                add_remove_connection(
                    passives_tree_specs,
                    passives_history_tracker,
                    selected_node_id,
                    node_id,
                );
            }
            None => selected_node.set(Some(node_id)),
        },
        ToolMode::Add => {}
    }
}

fn add_remove_connection(
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    passives_history_tracker: RwSignal<HistoryTracker<PassivesTreeSpecs>>,
    from: PassiveNodeId,
    to: PassiveNodeId,
) {
    passives_tree_specs.update(|passives_tree_specs| {
        if let Some((index, _)) =
            passives_tree_specs
                .connections
                .iter()
                .enumerate()
                .find(|(_, connection)| {
                    (connection.from == from && connection.to == to)
                        || (connection.from == to && connection.to == from)
                })
        {
            passives_tree_specs.connections.remove(index);
        } else {
            passives_tree_specs
                .connections
                .push(PassiveConnection { from, to });
        }
    });
    record_history(passives_history_tracker, passives_tree_specs);
}

fn add_node(
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    passives_history_tracker: RwSignal<HistoryTracker<PassivesTreeSpecs>>,
    mouse_position: RwSignal<(f64, f64)>,
) -> PassiveNodeId {
    let node_id: String = uuid::Uuid::new_v4().into();
    passives_tree_specs.update(|passives_tree_specs| {
        let (x, y) = mouse_position_to_node_position(mouse_position.get_untracked());

        passives_tree_specs.nodes.insert(
            node_id.clone(),
            PassiveNodeSpecs {
                name: "New Node".into(),
                icon: "passives/XXX.svg".into(),
                x,
                y,
                ..Default::default()
            },
        );
    });
    record_history(passives_history_tracker, passives_tree_specs);
    node_id
}

fn paste_node(
    node_id: &PassiveNodeId,
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    passives_history_tracker: RwSignal<HistoryTracker<PassivesTreeSpecs>>,
    clipboard_node: RwSignal<Option<PassiveNodeSpecs>>,
) {
    if let Some(clipboard_node) = clipboard_node.get_untracked() {
        if let Some(node_specs) = passives_tree_specs.write().nodes.get_mut(node_id) {
            *node_specs = PassiveNodeSpecs {
                x: node_specs.x,
                y: node_specs.y,
                ..clipboard_node
            };
        }
        record_history(passives_history_tracker, passives_tree_specs);
    }
}

fn delete_node(
    node_id: &PassiveNodeId,
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    passives_history_tracker: RwSignal<HistoryTracker<PassivesTreeSpecs>>,
) {
    passives_tree_specs.update(|passives_tree_specs| {
        passives_tree_specs
            .connections
            .retain(|connection| connection.from != *node_id && connection.to != *node_id);
        passives_tree_specs.nodes.remove_entry(node_id);
    });
    record_history(passives_history_tracker, passives_tree_specs);
}

fn record_history(
    passives_history_tracker: RwSignal<HistoryTracker<PassivesTreeSpecs>>,
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
) {
    passives_history_tracker
        .write()
        .push(passives_tree_specs.get_untracked());
}

fn undo_history(
    passives_history_tracker: RwSignal<HistoryTracker<PassivesTreeSpecs>>,
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
) {
    if let Some(specs) = passives_history_tracker.write().undo() {
        passives_tree_specs.set(specs.clone());
    }
}

fn redo_history(
    passives_history_tracker: RwSignal<HistoryTracker<PassivesTreeSpecs>>,
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
) {
    if let Some(specs) = passives_history_tracker.write().redo() {
        passives_tree_specs.set(specs.clone());
    }
}
