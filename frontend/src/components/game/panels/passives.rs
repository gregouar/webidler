use leptos::{html::*, prelude::*};

use std::collections::HashMap;
use std::sync::Arc;

use shared::{
    data::passive::{PassiveConnection, PassiveNodeId, PassiveNodeSpecs, PassiveNodeType},
    messages::client::PurchasePassiveMessage,
};

use crate::{
    assets::img_asset,
    components::shared::tooltips::effects_tooltip::formatted_effects_list,
    components::{
        game::game_context::GameContext,
        shared::tooltips::effects_tooltip,
        ui::{
            buttons::CloseButton,
            menu_panel::{MenuPanel, PanelTitle},
            pannable::Pannable,
            tooltip::{DynamicTooltipContext, DynamicTooltipPosition},
        },
        websocket::WebsocketContext,
    },
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PurchaseStatus {
    Inactive,
    Purchaseable,
    Purchased,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetaStatus {
    Normal,
    Locked,
    Ascended,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeStatus {
    pub purchase_status: PurchaseStatus,
    pub meta_status: MetaStatus,
}

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
            let meta_status = node_meta_status(node_level.get(), node_specs.locked);

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
            conn.send(
                &PurchasePassiveMessage {
                    node_id: node_id.clone(),
                }
                .into(),
            );
        }
    };

    view! { <Node node_specs node_status node_level on_click=purchase show_upgrade=false /> }
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

pub fn node_meta_status(node_level: u8, locked: bool) -> MetaStatus {
    if node_level > 0 {
        if locked && node_level == 1 {
            MetaStatus::Normal
        } else {
            MetaStatus::Ascended
        }
    } else if locked {
        MetaStatus::Locked
    } else {
        MetaStatus::Normal
    }
}

fn status_color(purchase_status: PurchaseStatus, meta_status: MetaStatus) -> &'static str {
    match (purchase_status, meta_status) {
        (PurchaseStatus::Inactive, MetaStatus::Normal) => "gray",
        (PurchaseStatus::Purchaseable, MetaStatus::Normal) => "darkgoldenrod",
        (PurchaseStatus::Purchased, MetaStatus::Normal) => "gold",

        (PurchaseStatus::Inactive, MetaStatus::Ascended) => "teal",
        (PurchaseStatus::Purchaseable, MetaStatus::Ascended) => "darkcyan",
        (PurchaseStatus::Purchased, MetaStatus::Ascended) => "cyan",

        (_, MetaStatus::Locked) => "red",
    }
}

#[component]
pub fn Node(
    node_specs: PassiveNodeSpecs,
    node_status: Memo<NodeStatus>,
    node_level: Memo<u8>,
    show_upgrade: bool,
    on_click: impl Fn() + Send + Sync + 'static,
) -> impl IntoView {
    let fill = match node_specs.node_type {
        PassiveNodeType::Attack => "#8b1e1e",
        PassiveNodeType::Life => "#386641",
        PassiveNodeType::Spell => "#533ea9",
        PassiveNodeType::Armor => "#5e5e5e",
        PassiveNodeType::Critical => "#ea6110",
        PassiveNodeType::Mana => "#3e5ba9",
        PassiveNodeType::Gold => "goldenrod",
        PassiveNodeType::Physical => "#2e2929ff",
        PassiveNodeType::Poison => "#98bb1bff",
        PassiveNodeType::Fire => "#da5011ff",
        PassiveNodeType::Storm => "#dac611ff",
        PassiveNodeType::Status => "#3ea9a4ff",
        PassiveNodeType::Utility => "#973ea9ff",
    };

    let node_specs = Arc::new(node_specs);

    let show_tooltip = {
        let tooltip_context = expect_context::<DynamicTooltipContext>();
        let node_specs = node_specs.clone();
        move || {
            let node_specs = node_specs.clone();
            tooltip_context.set_content(
                move || {
                    let node_specs = node_specs.clone();
                    view! { <NodeTooltip node_specs node_level show_upgrade /> }.into_any()
                },
                DynamicTooltipPosition::Auto,
            );
        }
    };

    let hide_tooltip = {
        let tooltip_context = expect_context::<DynamicTooltipContext>();
        move || tooltip_context.hide()
    };

    let icon_asset = img_asset(&node_specs.icon);

    let stroke = move || {
        let status = node_status.get();
        status_color(status.purchase_status, status.meta_status)
    };

    let filter = move || {
        let status = node_status.get();
        match (status.purchase_status, status.meta_status) {
            (PurchaseStatus::Inactive, MetaStatus::Normal) => "",
            (PurchaseStatus::Purchaseable, MetaStatus::Normal) => {
                "drop-shadow(0 0 2px darkgoldenrod)"
            }
            (PurchaseStatus::Purchased, MetaStatus::Normal) => "drop-shadow(0 0 4px gold)",

            (PurchaseStatus::Inactive, MetaStatus::Ascended) => "drop-shadow(0 0 2px cyan)",
            (PurchaseStatus::Purchaseable, MetaStatus::Ascended) => "drop-shadow(0 0 4px cyan)",
            (PurchaseStatus::Purchased, MetaStatus::Ascended) => "drop-shadow(0 0 6px cyan)",

            (_, MetaStatus::Locked) => "drop-shadow(0 0 2px red)",
        }
    };

    let class_style = move || {
        let status = node_status.get();
        match (status.purchase_status, status.meta_status) {
            (PurchaseStatus::Purchaseable, _) => {
                "saturate-50 cursor-pointer group active:brightness-50"
            }
            (_, MetaStatus::Locked) => "saturate-50 brightness-50",
            (PurchaseStatus::Inactive, _) => "saturate-50 brightness-50",
            _ => "",
        }
    };

    let icon_filter = move || {
        let status = node_status.get();
        match (status.purchase_status, status.meta_status) {
            (PurchaseStatus::Purchaseable, _) => "",
            (_, MetaStatus::Locked) => "brightness(0.3) saturate(0.5)",
            _ => "drop-shadow(2px 2px 2px black)",
        }
    };

    view! {
        <g
            transform=format!("translate({}, {})", node_specs.x * 10.0, -node_specs.y * 10.0)

            on:click=move |_| {
                let status = node_status.get();
                if status.purchase_status == PurchaseStatus::Purchaseable {
                    on_click();
                }
            }

            on:mousedown=|ev| ev.stop_propagation()

            on:touchstart={
                let show_tooltip = show_tooltip.clone();
                move |_| { show_tooltip() }
            }
            on:contextmenu=move |ev| {
                ev.prevent_default();
            }

            on:mouseenter=move |_| show_tooltip()
            on:mouseleave=move |_| hide_tooltip()

            class=class_style
        >
            <circle
                r=20 + node_specs.size * 5
                fill=fill
                stroke=stroke
                stroke-width="3"
                filter=filter
            />

            <circle r=20 + node_specs.size * 5 fill="url(#node-inner-gradient)" />

            <image
                filter="drop-shadow(2px 2px 2px black)"
                href=icon_asset
                x=-(24 + node_specs.size as i32 * 10) / 2
                y=-(24 + node_specs.size as i32 * 10) / 2
                width=24 + node_specs.size * 10
                height=24 + node_specs.size * 10
                class="group-active:scale-90 group-active:brightness-100"
                style=move || { format!("pointer-events: none; filter: {}", icon_filter()) }
            />
        </g>
    }
}

#[component]
pub fn Connection(
    connection: PassiveConnection,
    nodes_specs: Arc<HashMap<String, PassiveNodeSpecs>>,
    amount_connections: Memo<usize>,
    node_levels: Memo<(u8, u8)>,
) -> impl IntoView {
    let from_node = nodes_specs.get(&connection.from).cloned();
    let to_node = nodes_specs.get(&connection.to).cloned();

    view! {
        {if let (Some(from), Some(to)) = (from_node, to_node) {
            let (from_level, to_level) = node_levels.get_untracked();
            let from_status = node_meta_status(from_level, from.locked);
            let to_status = node_meta_status(to_level, to.locked);
            let purchase_status = move || match amount_connections.get() {
                2 => PurchaseStatus::Purchased,
                1 => PurchaseStatus::Purchaseable,
                _ => PurchaseStatus::Inactive,
            };
            let color = move |status| {
                match purchase_status() {
                    PurchaseStatus::Inactive => "gray",
                    x => status_color(x, status),
                }
            };
            let from_color = move || { color(from_status) };
            let to_color = move || { color(to_status) };
            let dasharray = move || if amount_connections.get() == 2 { "none" } else { "4 3" };
            let width = move || if amount_connections.get() == 2 { "3" } else { "2" };
            let gradient_id = format!("{}-{}", connection.from, connection.to);
            Some(

                view! {
                    <linearGradient
                        id=gradient_id.clone()
                        gradientUnits="userSpaceOnUse"
                        x1=from.x * 10.0
                        y1=-from.y * 10.0
                        x2=to.x * 10.0
                        y2=-to.y * 10.0
                    >
                        <stop offset="0%" stop-color=from_color />
                        <stop offset="100%" stop-color=to_color />
                    </linearGradient>
                    <line
                        x1=from.x * 10.0
                        y1=-from.y * 10.0
                        x2=to.x * 10.0
                        y2=-to.y * 10.0
                        filter=move || {
                            if amount_connections.get() == 2 {
                                match (from_status, to_status) {
                                    (MetaStatus::Ascended, MetaStatus::Ascended) => {
                                        "drop-shadow(0 0 2px cyan)"
                                    }
                                    _ => "drop-shadow(0 0 2px gold)",
                                }
                            } else {
                                ""
                            }
                        }
                        stroke=format!("url(#{gradient_id})")
                        stroke-dasharray=dasharray
                        stroke-linecap="round"
                        stroke-width=width
                    />
                },
            )
        } else {
            None
        }}
    }
}

#[component]
fn NodeTooltip(
    node_specs: Arc<PassiveNodeSpecs>,
    node_level: Memo<u8>,
    show_upgrade: bool,
) -> impl IntoView {
    let effects_text = {
        let node_specs = node_specs.clone();
        move || formatted_effects_list((&node_specs.aggregate_effects(node_level.get())).into())
    };

    let node_specs_locked = node_specs.locked;
    let max_upgrade_level = node_specs.max_upgrade_level;
    let triggers_text: Vec<_> = node_specs.triggers.iter().map(|trigger| view! { <li class="text-blue-400 text-sm leading-snug">{trigger.description.clone()}</li> }).collect();

    let is_locked = move || node_specs_locked && node_level.get() == 0;

    let locked_text = move || {
        is_locked().then(|| {
            view! {
                <hr class="border-t border-gray-700" />
                <ul class="list-none space-y-1">
                    <li class="text-blue-400 text-sm leading-snug text-red-500">"Locked"</li>
                </ul>
            }
        })
    };

    let upgrade_text = {
        let upgrade_effects = node_specs.upgrade_effects.clone();
        move || {
            if !show_upgrade {
                None
            } else if is_locked() {
                Some(
                    view! {
                        <hr class="border-t border-gray-700" />
                        <ul>
                            <li>
                                <span class="text-sm text-gray-400 leading-snug">
                                    "Ascend to Unlock"
                                </span>
                            </li>
                        </ul>
                    }
                    .into_any(),
                )
            } else if !upgrade_effects.is_empty() {
                let max_level = node_level.get() >= max_upgrade_level.unwrap_or(u8::MAX);
                Some(
                    view! {
                        <hr class="border-t border-gray-700" />
                        <p class="text-sm text-gray-400 leading-snug">
                            "Level: " <span class="text-white">{node_level}</span>
                            {max_upgrade_level
                                .map(|max_upgrade_level| format!("/{}", max_upgrade_level))
                                .unwrap_or_default()}
                            {if max_level {
                                view! {
                                    " | "
                                    <span class="text-cyan-300">"Ascended"</span>
                                }
                                    .into_any()
                            } else {
                                view! {
                                    " | Ascend Cost: "
                                    <span class="text-cyan-300">"1 Power Shard"</span>
                                }
                                    .into_any()
                            }}
                        </p>
                        {(!max_level)
                            .then(|| {
                                view! {
                                    <hr class="border-t border-gray-700" />
                                    <ul>
                                        <li>
                                            <span class="text-sm text-gray-400 leading-snug">
                                                "Ascend to get:"
                                            </span>
                                        </li>
                                        {effects_tooltip::formatted_effects_list(
                                            upgrade_effects.clone(),
                                        )}
                                    </ul>
                                }
                            })}
                    }
                    .into_any(),
                )
            } else {
                None
            }
        }
    };

    view! {
        <div class="
        max-w-xs p-4 rounded-xl border border-teal-700 ring-2 ring-teal-500 
        shadow-md shadow-teal-700 bg-gradient-to-br from-gray-800 via-gray-900 to-black space-y-2
        ">
            <strong class="text-lg font-bold text-teal-300">{node_specs.name.clone()}</strong>
            <hr class="border-t border-gray-700" />
            <ul class="list-none space-y-1">{triggers_text}{effects_text}</ul>
            {locked_text}
            {upgrade_text}
        </div>
    }
}
