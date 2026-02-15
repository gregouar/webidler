use itertools::Itertools;
use leptos::{html::*, prelude::*};

use std::sync::Arc;

use shared::data::passive::{
    PassiveConnection, PassiveNodeSpecs, PassiveNodeType, PassivesTreeSpecs,
};

use crate::{
    assets::img_asset,
    components::{
        accessibility::AccessibilityContext,
        shared::tooltips::{
            effects_tooltip::{self, formatted_effects_list},
            trigger_tooltip::{self, format_trigger},
        },
        ui::tooltip::{DynamicTooltipContext, DynamicTooltipPosition},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum PurchaseStatus {
    #[default]
    Inactive,
    Purchaseable,
    Purchased,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum MetaStatus {
    #[default]
    Normal,
    Locked,
    Ascended,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NodeStatus {
    pub purchase_status: PurchaseStatus,
    pub meta_status: MetaStatus,
}

pub fn node_meta_status(
    node_level: u8,
    locked: bool,
    // max_upgrade_level: Option<u8>,
) -> MetaStatus {
    if node_level > 0 {
        // if locked && node_level == 1 && max_upgrade_level.unwrap_or_default() > 0 {
        //     MetaStatus::Normal
        // } else {
        //     MetaStatus::Ascended
        // }
        MetaStatus::Ascended
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

fn node_size(node_specs: &PassiveNodeSpecs) -> i32 {
    node_specs.size as i32 * 7
}

#[component]
pub fn Node(
    node_specs: PassiveNodeSpecs,
    node_status: Memo<NodeStatus>,
    node_level: Memo<u8>,
    show_upgrade: bool,
    on_click: impl Fn() + Send + Sync + 'static,
    on_right_click: impl Fn() + Send + Sync + 'static,
    #[prop(optional, into)] search_node: Option<Signal<Option<String>>>,
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
    let tooltip_context: Option<DynamicTooltipContext> = use_context();
    let accessibility: Option<AccessibilityContext> = use_context();

    let node_text = node_text(&node_specs).to_lowercase();

    let highlight = Memo::new(move |_| {
        search_node
            .map(|search_node| match search_node.read().as_ref() {
                Some(searched_node) if !searched_node.is_empty() => {
                    node_text.contains(&searched_node.to_lowercase())
                }
                _ => false,
            })
            .unwrap_or_default()
    });

    let node_status = move || node_status.try_get().unwrap_or_default();

    let node_specs = Arc::new(node_specs);
    let show_tooltip = {
        let node_specs = node_specs.clone();
        move || {
            let node_specs = node_specs.clone();
            if let Some(tooltip_context) = tooltip_context {
                tooltip_context.set_content(
                    move || {
                        let node_specs = node_specs.clone();
                        view! { <NodeTooltip node_specs node_level show_upgrade /> }.into_any()
                    },
                    DynamicTooltipPosition::Auto,
                );
            }
        }
    };

    let hide_tooltip = {
        move || {
            if let Some(tooltip_context) = tooltip_context {
                tooltip_context.hide()
            }
        }
    };

    let stroke = move || {
        let status = node_status();
        status_color(status.purchase_status, status.meta_status)
    };

    let shadow_class = move || {
        let status = node_status();
        match (status.purchase_status, status.meta_status) {
            (PurchaseStatus::Inactive, MetaStatus::Normal) => "",
            (PurchaseStatus::Purchaseable, MetaStatus::Normal) => {
                "xl:drop-shadow-[0_0_2px_darkgoldenrod]"
            }
            (PurchaseStatus::Purchased, MetaStatus::Normal) => "xl:drop-shadow-[0_0_4px_gold]",

            (PurchaseStatus::Inactive, MetaStatus::Ascended) => "xl:drop-shadow-[0_0_2px_cyan]",
            (PurchaseStatus::Purchaseable, MetaStatus::Ascended) => {
                "xl:drop-shadow-[0_0_4px_cyan])"
            }
            (PurchaseStatus::Purchased, MetaStatus::Ascended) => "xl:drop-shadow-[0_0_6px_cyan]",

            (PurchaseStatus::Purchased, MetaStatus::Locked) => "xl:drop-shadow-[0_0_6px_red]",
            (_, MetaStatus::Locked) => "xl:drop-shadow-[0_0_2px_red]",
        }
    };

    let class_style = move || {
        let status = node_status();
        match (status.purchase_status, status.meta_status) {
            (PurchaseStatus::Purchaseable, _) => {
                "saturate-50 cursor-pointer group active:brightness-50 pointer-events: none"
            }
            (PurchaseStatus::Inactive, MetaStatus::Locked) => {
                "saturate-50 brightness-30 pointer-events: none"
            }
            (PurchaseStatus::Inactive, _) => "saturate-20 brightness-30 pointer-events: none",
            _ => "pointer-events: none",
        }
    };

    let icon_filter = move || {
        let status = node_status();
        match (status.purchase_status, status.meta_status) {
            (PurchaseStatus::Purchaseable, _) => "",
            (PurchaseStatus::Inactive, MetaStatus::Locked) => "brightness(0.3) saturate(0.5)",
            _ => "",
        }
    };

    let invert_filter = match node_specs.socket {
        true => "",
        false => "invert(1)",
    };

    let node_size = node_size(&node_specs);

    view! {
        <g
            transform=format!("translate({}, {})", node_specs.x * 10.0, -node_specs.y * 10.0)

            on:click=move |ev| {
                ev.stop_propagation();
                let status = node_status();
                if status.purchase_status == PurchaseStatus::Purchaseable {
                    on_click();
                }
            }

            on:mousedown=|ev| {
                if ev.button() == 0 {
                    ev.stop_propagation()
                }
            }

            on:touchstart={
                let show_tooltip = show_tooltip.clone();
                move |_| { show_tooltip() }
            }
            on:contextmenu=move |ev| {
                ev.prevent_default();
                if let Some(accessibility) = accessibility && !accessibility.is_on_mobile() {
                    on_right_click();
                }
            }

            on:mouseenter=move |_| show_tooltip()
            on:mouseleave=move |_| hide_tooltip()
        >
            <g class=class_style>
                {node_specs
                    .root_node
                    .then(|| {
                        view! {
                            <circle
                                r=20 + node_size + 5
                                fill="black"
                                stroke=stroke
                                stroke-width="2"
                                class=shadow_class
                            />
                        }
                    })}
                <circle
                    r=20 + node_size
                    fill=fill
                    stroke=stroke
                    stroke-width="3"
                    class=shadow_class
                /> <circle r=20 + node_size fill="url(#node-inner-gradient)" />
                {(node_specs.socket)
                    .then(|| {
                        view! {
                            <circle r=20 + node_size fill="url(#socket-outer-gradient)" />
                            <circle
                                r=14 + node_size
                                fill="url(#socket-inner-gradient)"
                                stroke="none"
                            />
                            <text
                                text-anchor="middle"
                                dominant-baseline="central"
                                fill="rgba(255,255,255,0.4)"
                                font-size="16"
                            >
                                "+"
                            </text>
                        }
                    })}
                {
                    let node_specs = node_specs.clone();
                    move || {
                        (!node_specs.icon.is_empty())
                            .then(|| {
                                view! {
                                    <image
                                        href=img_asset(&node_specs.icon)
                                        x=-12 - node_size
                                        y=-12 - node_size
                                        width=24 + node_size * 2
                                        height=24 + node_size * 2
                                        class="group-active:scale-90 group-active:brightness-100
                                        xl:drop-shadow-[2px_2px_2px_black]"
                                        style=move || {
                                            format!(
                                                "pointer-events: none;
                                            filter: {} {}",
                                                icon_filter(),
                                                invert_filter,
                                            )
                                        }
                                    />
                                }
                            })
                    }
                }
                {(node_specs.socket)
                    .then(|| {
                        view! {
                            <circle
                                r=14 + node_size
                                fill="none"
                                stroke="rgb(80, 80, 80)"
                                stroke-width="1"
                            />
                        }
                    })}

            </g>

            {move || {
                highlight
                    .get()
                    .then(|| {
                        view! {
                            <circle
                                r=22 + node_size
                                class="
                                animate-pulse
                                fill-fuchsia-400/40
                                stroke-fuchsia-500
                                stroke-[2]
                                pointer-events-none
                                "
                            />
                        }
                    })
            }}
        </g>
    }
}

#[component]
pub fn Connection(
    connection: PassiveConnection,
    // TODO: Could we avoid passing the whole thing?
    passives_tree_specs: RwSignal<PassivesTreeSpecs>,
    amount_connections: Memo<usize>,
    node_levels: Memo<(u8, u8)>,
) -> impl IntoView {
    let from_node = {
        let node_id = connection.from;
        move || passives_tree_specs.read().nodes.get(&node_id).cloned()
    };
    let to_node = {
        let node_id = connection.to;
        move || passives_tree_specs.read().nodes.get(&node_id).cloned()
    };

    view! {
        {move || {
            if let (Some(from), Some(to)) = (from_node(), to_node()) {
                let from_status = move || node_meta_status(node_levels.get().0, from.locked);
                let to_status = move || node_meta_status(node_levels.get().1, to.locked);
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
                let from_color = move || { color(from_status()) };
                let to_color = move || { color(to_status()) };
                let dasharray = move || if amount_connections.get() == 2 { "none" } else { "4 3" };
                let width = move || if amount_connections.get() == 2 { "3" } else { "2" };
                let gradient_id = format!("{}-{}", connection.from, connection.to);
                Some(
                    // from.max_upgrade_level,
                    // to.max_upgrade_level,

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
                            // class=move || {
                            // if amount_connections.get() == 2 {
                            // match (from_status(), to_status()) {
                            // (MetaStatus::Ascended, MetaStatus::Ascended) => {
                            // "xl:drop-shadow-[0_0_2px_cyan]"
                            // }
                            // _ => "xl:drop-shadow-[0_0_2px_gold]",
                            // }
                            // } else {
                            // ""
                            // }
                            // }
                            style="pointer-events: none"
                            stroke=format!("url(#{gradient_id})")
                            stroke-dasharray=dasharray
                            stroke-linecap="round"
                            stroke-width=width
                        />
                    },
                )
            } else {
                None
            }
        }}
    }
}

#[component]
pub fn NodeTooltip(
    node_specs: Arc<PassiveNodeSpecs>,
    node_level: Memo<u8>,
    show_upgrade: bool,
) -> impl IntoView {
    view! {
        <div class="
        max-w-xs p-4 rounded-xl border border-teal-700 ring-2 ring-teal-500 
        shadow-md shadow-teal-700 bg-gradient-to-br from-gray-800 via-gray-900 to-black space-y-2
        ">
            <NodeTooltipContent node_specs node_level show_upgrade />
        </div>
    }
}

#[component]
pub fn NodeTooltipContent(
    node_specs: Arc<PassiveNodeSpecs>,
    node_level: Memo<u8>,
    show_upgrade: bool,
) -> impl IntoView {
    let node_level = move || node_level.try_get().unwrap_or_default();

    let effects_text = {
        let node_specs = node_specs.clone();
        move || formatted_effects_list((&node_specs.aggregate_effects(node_level())).into())
    };

    let node_specs_locked = node_specs.locked;
    let max_upgrade_level = node_specs.max_upgrade_level;
    // let triggers_text: Vec<_> = node_specs.triggers.iter().map(|trigger| view! { <li class="text-blue-400 text-sm leading-snug">{trigger.description.clone()}</li> }).collect();
    let triggers_text: Vec<_> = node_specs
        .triggers
        .clone()
        .into_iter()
        .map(format_trigger)
        .collect();

    let is_locked = move || node_specs_locked && node_level() == 0;

    let starting_node_text = (node_specs.root_node).then(|| {
        view! {
            <ul class="list-none space-y-1">
                <li class="text-gray-400 text-sm leading-snug">"Root Node"</li>
            </ul>
            <hr class="border-t border-gray-700" />
        }
    });
    let locked_text = move || {
        is_locked().then(|| {
            view! {
                <hr class="border-t border-gray-700" />
                <ul class="list-none space-y-1">
                    <li class="text-red-500 text-sm leading-snug">"Locked"</li>
                </ul>
            }
        })
    };

    let socket_text = {
        (node_specs.socket).then(|| {
            view! {
                {(node_specs.effects.is_empty() && node_specs.triggers.is_empty())
                    .then(|| {
                        view! {
                            <ul class="list-none space-y-1">
                                <li class="text-sm text-gray-400 leading-snug italic">"Empty"</li>
                            </ul>
                        }
                    })}
                <hr class="border-t border-gray-700" />
                <ul>
                    <li class="text-sm text-gray-400 leading-snug">"Ascend to Socket Rune"</li>
                </ul>
            }
            .into_any()
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
                let max_level = node_level() >= max_upgrade_level.unwrap_or(u8::MAX);
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
                                    <ul class="text-xs xl:text-sm">
                                        <li>
                                            <span class="text-gray-400 leading-snug">
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
        <strong class="text-lg font-bold text-teal-300">
            <ul class="list-none space-y-1 mb-2">
                <li class="leading-snug whitespace-pre-line">{node_specs.name.clone()}</li>
            </ul>
        </strong>
        <hr class="border-t border-gray-700" />
        {starting_node_text}
        <ul class="list-none space-y-1 text-xs xl:text-sm">{effects_text}{triggers_text}</ul>
        {socket_text}
        {locked_text}
        {upgrade_text}
    }
}

pub fn node_text(node_specs: &PassiveNodeSpecs) -> String {
    format!(
        "{} {} {}",
        node_specs.name,
        node_specs
            .effects
            .iter()
            .map(effects_tooltip::format_stat)
            .join(" "),
        node_specs
            .triggers
            .iter()
            .cloned()
            .map(trigger_tooltip::trigger_text)
            .join(" ")
    )
}
