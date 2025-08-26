use leptos::{html::*, prelude::*};
use shared::data::item_affix::AffixEffectScope;

use std::collections::HashMap;
use std::sync::Arc;

use shared::data::passive::{PassiveConnection, PassiveNodeId, PassiveNodeSpecs, PassiveNodeType};
use shared::messages::client::PurchasePassiveMessage;

use crate::assets::img_asset;
use crate::components::{
    game::{game_context::GameContext, tooltips::effects_tooltip::formatted_effects_list},
    ui::{
        buttons::CloseButton,
        menu_panel::MenuPanel,
        pannable::Pannable,
        tooltip::{DynamicTooltipContext, DynamicTooltipPosition},
    },
    websocket::WebsocketContext,
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
    // TODO: bool memo locked, use also in tooltip

    let node_status = Memo::new({
        let game_context = expect_context::<GameContext>();
        let node_id = node_id.clone();

        move |_| {
            let ascend_level = game_context
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

    view! { <Node node_specs node_status on_click=purchase /> }
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

    view! {
        <Connection
            connection=connection
            nodes_specs=nodes_specs
            amount_connections=amount_connections
        />
    }
}

#[component]
pub fn Node(
    node_specs: PassiveNodeSpecs,
    node_status: Memo<NodeStatus>,
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
        PassiveNodeType::Status => "#3ea9a4ff",
    };

    // TODO: bool memo locked, use also in tooltip

    let node_specs = Arc::new(node_specs);

    let show_tooltip = {
        let tooltip_context = expect_context::<DynamicTooltipContext>();
        let node_specs = node_specs.clone();
        move |_| {
            let node_specs = node_specs.clone();
            tooltip_context.set_content(
                move || {
                    let node_specs = node_specs.clone();
                    view! { <NodeTooltip node_specs=node_specs /> }.into_any()
                },
                DynamicTooltipPosition::Auto,
            );
        }
    };

    let hide_tooltip = {
        let tooltip_context = expect_context::<DynamicTooltipContext>();
        move |_| tooltip_context.hide()
    };

    let icon_asset = img_asset(&node_specs.icon);

    let stroke = move || {
        let status = node_status.get();
        match (status.purchase_status, status.meta_status) {
            (PurchaseStatus::Inactive, MetaStatus::Normal) => "gray",
            (PurchaseStatus::Purchaseable, MetaStatus::Normal) => "darkgoldenrod",
            (PurchaseStatus::Purchased, MetaStatus::Normal) => "gold",

            (PurchaseStatus::Inactive, MetaStatus::Ascended) => "teal",
            (PurchaseStatus::Purchaseable, MetaStatus::Ascended) => "darkcyan",
            (PurchaseStatus::Purchased, MetaStatus::Ascended) => "cyan",

            (_, MetaStatus::Locked) => "red",
        }
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

    let pointer_style = move || {
        let status = node_status.get();
        match (status.purchase_status, status.meta_status) {
            (PurchaseStatus::Purchaseable, _) => "cursor: pointer;",
            _ => "",
        }
    };

    let class_style = move || {
        let status = node_status.get();
        match (status.purchase_status, status.meta_status) {
            (PurchaseStatus::Purchaseable, _) => "saturate-50",
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
            on:mouseenter=show_tooltip
            on:mouseleave=hide_tooltip
            on:click=move |_| {
                let status = node_status.get();
                if status.purchase_status == PurchaseStatus::Purchaseable {
                    on_click();
                }
            }
            style=pointer_style
            class=class_style
        >
            <circle
                r=20 + node_specs.size * 10
                fill=fill
                stroke=stroke
                stroke-width="3"
                filter=filter
            />

            <circle r=20 + node_specs.size * 10 fill="url(#node-inner-gradient)" />

            <image
                filter="drop-shadow(2px 2px 2px black)"
                href=icon_asset
                x=-(24 + node_specs.size as i32 * 20) / 2
                y=-(24 + node_specs.size as i32 * 20) / 2
                width=24 + node_specs.size * 20
                height=24 + node_specs.size * 20
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
) -> impl IntoView {
    let from_node = nodes_specs.get(&connection.from).cloned();
    let to_node = nodes_specs.get(&connection.to).cloned();

    view! {
        {if let (Some(from), Some(to)) = (from_node, to_node) {
            let stroke_color = {
                move || match amount_connections.get() {
                    2 => "gold",
                    1 => "darkgoldenrod",
                    _ => "gray",
                }
            };
            let dasharray = move || if amount_connections.get() == 2 { "none" } else { "4 3" };
            let width = move || if amount_connections.get() == 2 { "3" } else { "2" };
            Some(
                view! {
                    <line
                        x1=from.x * 10.0
                        y1=-from.y * 10.0
                        x2=to.x * 10.0
                        y2=-to.y * 10.0
                        filter=move || {
                            if amount_connections.get() == 2 {
                                "drop-shadow(0 0 2px gold)"
                            } else {
                                ""
                            }
                        }
                        stroke=stroke_color
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
pub fn NodeTooltip(node_specs: Arc<PassiveNodeSpecs>) -> impl IntoView {
    let effects = formatted_effects_list(node_specs.effects.clone(), AffixEffectScope::Global);
    let triggers: Vec<_> = node_specs.triggers.iter().map(|trigger| view! { <li class="text-blue-400 text-sm leading-snug">{trigger.description.clone()}</li> }).collect();

    view! {
        <div class="
        max-w-xs p-4 rounded-xl border border-teal-700 ring-2 ring-teal-500 
        shadow-md shadow-teal-700 bg-gradient-to-br from-gray-800 via-gray-900 to-black space-y-2
        ">
            <strong class="text-lg font-bold text-teal-300">{node_specs.name.clone()}</strong>
            <hr class="border-t border-gray-700" />
            <ul class="list-none space-y-1">{triggers}{effects}</ul>
        </div>
    }
}
