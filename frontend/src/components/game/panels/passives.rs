use leptos::{html::*, prelude::*, web_sys};
use shared::data::item_affix::AffixEffectScope;

use std::sync::Arc;

use shared::data::passive::{PassiveConnection, PassiveNodeId, PassiveNodeSpecs, PassiveNodeType};
use shared::messages::client::PurchasePassiveMessage;

use crate::assets::img_asset;
use crate::components::{
    game::{game_context::GameContext, tooltips::effects_tooltip::formatted_effects_list},
    ui::{
        buttons::CloseButton,
        menu_panel::MenuPanel,
        tooltip::{DynamicTooltipContext, DynamicTooltipPosition},
    },
    websocket::WebsocketContext,
};

#[derive(Debug, Clone, Copy, PartialEq)]
enum NodeStatus {
    Inactive,
    Purchaseable,
    Purchased,
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
pub fn PassiveSkillTree() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let offset = RwSignal::new((0.0, 0.0)); // for panning
    let dragging = RwSignal::new(None::<(f64, f64)>);
    let zoom = RwSignal::new(1.0f64);

    let on_mouse_down = move |ev: web_sys::MouseEvent| {
        dragging.set(Some((ev.client_x() as f64, ev.client_y() as f64)));
    };

    let on_mouse_up = move |_| dragging.set(None);

    let on_mouse_move = {
        move |ev: web_sys::MouseEvent| {
            if let Some((last_x, last_y)) = dragging.get() {
                let dx = ev.client_x() as f64 - last_x;
                let dy = ev.client_y() as f64 - last_y;
                offset.update(|(x, y)| {
                    *x += dx;
                    *y += dy;
                });
                dragging.set(Some((ev.client_x() as f64, ev.client_y() as f64)));
            }
        }
    };

    let on_wheel = move |ev: web_sys::WheelEvent| {
        ev.prevent_default();
        zoom.set((zoom.get() * if ev.delta_y() < 0.0 { 1.1 } else { 0.9 }).clamp(0.5, 3.0));
    };

    let points_available =
        Memo::new(move |_| game_context.player_resources.read().passive_points > 0);

    view! {
        <div
            on:wheel=on_wheel
            on:mousedown=on_mouse_down
            on:mouseup=on_mouse_up
            on:mousemove=on_mouse_move
            class="w-full aspect-[5/2] overflow-hidden bg-neutral-900"
        >
            // style="filter: drop-shadow(0 2px 4px black);"
            <svg
                width="100%"
                height="100%"
                viewBox="-500 -500 1000 1000"
                preserveAspectRatio="xMidYMid meet"
            >
                <defs>
                    <radialGradient id="node-inner-gradient" cx="50%" cy="50%" r="50%">
                        <stop offset="20%" stop-color="black" stop-opacity=0 />
                        <stop offset="70%" stop-color="black" stop-opacity=0.5 />
                        <stop offset="100%" stop-color="black" stop-opacity=0.8 />
                    </radialGradient>
                </defs>
                <g
                    transform=move || {
                        let (x, y) = offset.get();
                        format!("translate({x},{y}),scale({})", zoom.get())
                    }
                    filter="drop-shadow(0 2px 4px black)"
                >
                    <For
                        each=move || {
                            game_context.passives_tree_specs.read().connections.clone().into_iter()
                        }
                        key=|conn| (conn.from.clone(), conn.to.clone())
                        let(conn)
                    >
                        <Connection connection=conn />
                    </For>
                    <For
                        each=move || {
                            game_context.passives_tree_specs.read().nodes.clone().into_iter()
                        }
                        key=|(id, _)| id.clone()
                        let((id, node))
                    >
                        <Node node_id=id node_specs=node points_available=points_available />
                    </For>
                </g>
            </svg>
        </div>
    }
}

#[component]
fn Node(
    node_id: PassiveNodeId,
    node_specs: PassiveNodeSpecs,
    points_available: Memo<bool>,
) -> impl IntoView {
    let fill = match node_specs.node_type {
        PassiveNodeType::Attack => "#8b1e1e",
        PassiveNodeType::Life => "#386641",
        PassiveNodeType::Spell => "#533ea9",
        PassiveNodeType::Armor => "#5e5e5e",
        PassiveNodeType::Critical => "#ea6110",
        PassiveNodeType::Mana => "#3e5ba9",
        PassiveNodeType::Gold => "goldenrod",
    };

    let node_status = Memo::new({
        let game_context = expect_context::<GameContext>();
        let node_id = node_id.clone();

        move |_| {
            if game_context
                .passives_tree_state
                .read()
                .purchased_nodes
                .contains(&node_id)
            {
                NodeStatus::Purchased
            } else if points_available.get()
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
                NodeStatus::Purchaseable
            } else {
                NodeStatus::Inactive
            }
        }
    });

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

    let icon_asset = img_asset(&node_specs.icon);

    let stroke = move || match node_status.get() {
        NodeStatus::Inactive => "gray",
        NodeStatus::Purchaseable => "darkgoldenrod",
        NodeStatus::Purchased => "gold",
    };

    let filter = move || match node_status.get() {
        NodeStatus::Inactive => "",
        NodeStatus::Purchaseable => "",
        NodeStatus::Purchased => "drop-shadow(0 0 4px gold)",
    };

    view! {
        <g
            transform=format!("translate({}, {})", node_specs.x * 10.0, -node_specs.y * 10.0)
            on:mouseenter=show_tooltip
            on:mouseleave=hide_tooltip
            on:click=move |_| {
                if let NodeStatus::Purchaseable = node_status.get() {
                    purchase();
                }
            }
            style=move || {
                match node_status.get() {
                    NodeStatus::Inactive => "",
                    NodeStatus::Purchaseable => "cursor: pointer; ",
                    NodeStatus::Purchased => "",
                }
            }
            class=move || {
                match node_status.get() {
                    NodeStatus::Inactive => "saturate-50 brightness-50",
                    NodeStatus::Purchaseable => "saturate-50",
                    NodeStatus::Purchased => "",
                }
            }
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
                style="pointer-events: none"
            />
        </g>
    }
}

#[component]
fn Connection(connection: PassiveConnection) -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let from_node = game_context
        .passives_tree_specs
        .read_untracked()
        .nodes
        .get(&connection.from)
        .cloned();
    let to_node = game_context
        .passives_tree_specs
        .read_untracked()
        .nodes
        .get(&connection.to)
        .cloned();

    view! {
        {if let (Some(from), Some(to)) = (from_node, to_node) {
            let amount_connections = {
                let game_context = expect_context::<GameContext>();
                move || {
                    game_context
                        .passives_tree_state
                        .read()
                        .purchased_nodes
                        .contains(&connection.from) as usize
                        + game_context
                            .passives_tree_state
                            .read()
                            .purchased_nodes
                            .contains(&connection.to) as usize
                }
            };
            let stroke_color = {
                let amount_connections = amount_connections.clone();
                move || match amount_connections() {
                    2 => "gold",
                    1 => "darkgoldenrod",
                    _ => "gray",
                }
            };
            let dasharray = {
                let amount_connections = amount_connections.clone();
                move || if amount_connections() == 2 { "none" } else { "4 3" }
            };
            let width = {
                let amount_connections = amount_connections.clone();
                move || if amount_connections() == 2 { "3" } else { "2" }
            };
            Some(

                view! {
                    <line
                        x1=from.x * 10.0
                        y1=-from.y * 10.0
                        x2=to.x * 10.0
                        y2=-to.y * 10.0
                        filter=move || {
                            if amount_connections() == 2 { "drop-shadow(0 0 2px gold)" } else { "" }
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
