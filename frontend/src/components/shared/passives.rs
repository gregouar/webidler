use itertools::Itertools;
use leptos::{html::*, prelude::*};

use std::sync::Arc;

use shared::data::passive::{
    self, PassiveConnection, PassiveNodeSpecs, PassiveNodeType, PassivesTreeAscension,
    PassivesTreeSpecs, PurchasedNodes,
};

use crate::{
    assets::img_asset,
    components::{
        accessibility::AccessibilityContext,
        shared::tooltips::{
            effects_tooltip::{self, formatted_effects_list},
            trigger_tooltip::{self, format_trigger},
        },
        ui::{
            Separator,
            tooltip::{DynamicTooltipContext, DynamicTooltipPosition},
        },
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
                                        preserveAspectRatio="xMidYMid slice"
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
                let from_status = move || node_meta_status(
                    node_levels.try_get().unwrap_or_default().0,
                    from.locked,
                );
                let to_status = move || node_meta_status(
                    node_levels.try_get().unwrap_or_default().1,
                    to.locked,
                );
                let purchase_status = move || match amount_connections.try_get().unwrap_or_default()
                {
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
                let dasharray = move || {
                    if amount_connections.try_get().unwrap_or_default() == 2 {
                        "none"
                    } else {
                        "4 3"
                    }
                };
                let width = move || {
                    if amount_connections.try_get().unwrap_or_default() == 2 { "3" } else { "2" }
                };
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
        <div class="relative isolate max-w-xs text-center">
            <div
                class="pointer-events-none absolute inset-0"
                aria-hidden="true"
                style="filter: drop-shadow(0 10px 20px rgba(13,88,88,0.42)) drop-shadow(0 3px 5px rgba(0,0,0,0.45));"
            >
                <div
                    class="absolute inset-0 bg-black/90"
                    style="clip-path: polygon(10px 0, calc(100% - 10px) 0, 100% 10px, 100% calc(100% - 10px), calc(100% - 10px) 100%, 10px 100%, 0 calc(100% - 10px), 0 10px);"
                ></div>
            </div>
            <div
                class="relative overflow-hidden border border-teal-700/90 shadow-[inset_0_1px_0_rgba(240,215,159,0.16),inset_0_-1px_0_rgba(0,0,0,0.5)]"
                style="clip-path: polygon(10px 0, calc(100% - 10px) 0, 100% 10px, 100% calc(100% - 10px), calc(100% - 10px) 100%, 10px 100%, 0 calc(100% - 10px), 0 10px);
                background-image:
                    linear-gradient(180deg, rgba(214,177,102,0.05), rgba(0,0,0,0.2)),
                    radial-gradient(circle at 50% 20%, rgba(60,180,180,0.16), transparent 64%),
                    linear-gradient(180deg, rgba(92,212,204,0.12), transparent 34%),
                    linear-gradient(135deg, rgba(31,33,36,0.985), rgba(9,9,12,1));
                background-blend-mode: screen, soft-light, screen, normal;"
            >
                <div
                    class="pointer-events-none absolute inset-[1px] border border-teal-200/10"
                    style="clip-path: polygon(9px 0, calc(100% - 9px) 0, 100% 9px, 100% calc(100% - 9px), calc(100% - 9px) 100%, 9px 100%, 0 calc(100% - 9px), 0 9px);"
                ></div>
                <span class="pointer-events-none absolute inset-x-[6px] top-[2px] h-[2px] bg-gradient-to-r from-transparent via-[rgba(153,244,234,0.45)] to-transparent"></span>
                <span class="pointer-events-none absolute inset-y-[5px] left-[1px] w-[2px] bg-gradient-to-b from-transparent via-[rgba(153,244,234,0.35)] to-transparent"></span>
                <span class="pointer-events-none absolute inset-y-[5px] right-[1px] w-[2px] bg-gradient-to-b from-transparent via-[rgba(153,244,234,0.35)] to-transparent"></span>
                <div class="space-y-2 p-2 xl:p-4">
                    <NodeTooltipContent node_specs node_level show_upgrade />
                </div>
            </div>
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
    // let triggers_text: Vec<_> = node_specs.triggers.iter().map(|trigger| view! { <li class="text-blue-400 text-sm ">{trigger.description.clone()}</li> }).collect();
    let triggers_text: Vec<_> = node_specs
        .triggers
        .clone()
        .into_iter()
        .map(format_trigger)
        .collect();

    let is_locked = move || node_specs_locked && node_level() == 0;

    let starting_node_text = (node_specs.root_node).then(|| {
        view! {
            <ul class="list-none xl:space-y-1">
                <li class="text-gray-400 text-sm ">"Root Node"</li>
            </ul>
            <Separator />
        }
    });
    let locked_text = move || {
        is_locked().then(|| {
            view! {
                <Separator />
                <ul class="list-none xl:space-y-1">
                    <li class="text-red-500 text-sm ">"Locked"</li>
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
                            <ul class="list-none xl:space-y-1">
                                <li class="text-sm text-gray-400  italic">"Empty"</li>
                            </ul>
                        }
                    })}
                <Separator />
                <ul>
                    <li class="text-sm text-gray-400 ">"Ascend to Socket Rune"</li>
                </ul>
            }
            .into_any()
        })
    };

    let upgrade_text = {
        let node_specs = node_specs.clone();
        move || {
            if !show_upgrade {
                None
            } else if is_locked() {
                Some(
                    view! {
                        <Separator />
                        <ul>
                            <li>
                                <span class="text-sm text-gray-400 ">"Ascend to Unlock"</span>
                            </li>
                        </ul>
                    }
                    .into_any(),
                )
            } else if !node_specs.upgrade_effects.is_empty() {
                let max_level = node_level() >= max_upgrade_level.unwrap_or(u8::MAX);
                Some(
                    view! {
                        <Separator />
                        <p class="text-xs xl:text-sm text-gray-400 ">
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
                                    <span class="text-cyan-300">
                                        <span class="font-semibold">
                                            {node_specs.next_ascend_cost(node_level())}
                                        </span>
                                        " Power Shard(s)"
                                    </span>
                                }
                                    .into_any()
                            }}
                        </p>
                        {(!max_level)
                            .then(|| {
                                view! {
                                    <Separator />
                                    <ul class="text-xs xl:text-sm">
                                        <li>
                                            <span class="text-gray-400 ">"Ascend to get:"</span>
                                        </li>
                                        {effects_tooltip::formatted_effects_list(
                                            node_specs.upgrade_effects.clone(),
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
        <strong class="text-sm xl:text-base font-bold text-teal-300 font-display text-shadow-md/80">
            <ul class="list-none xl:space-y-1 mb-2">
                <li class=" whitespace-pre-line">{node_specs.name.clone()}</li>
            </ul>
        </strong>
        <Separator />
        {starting_node_text}
        <ul class="list-none xl:space-y-1 text-xs xl:text-sm">{effects_text}{triggers_text}</ul>
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

#[component]
pub fn PassiveSkillStats(
    #[prop(into)] passives_tree_specs: Signal<PassivesTreeSpecs>,
    #[prop(into)] passives_tree_ascension: Signal<PassivesTreeAscension>,
    #[prop(into)] purchased_nodes: Signal<PurchasedNodes>,
) -> impl IntoView {
    let stats = Memo::new(move |_| {
        passives_tree_specs.with(|passives_tree_specs| {
            passives_tree_ascension.with(|passives_tree_ascension| {
                purchased_nodes.with(|purchased_nodes| {
                    passive::generate_effects_map_from_passives(
                        passives_tree_specs,
                        passives_tree_ascension,
                        purchased_nodes,
                    )
                })
            })
        })
    });

    let (panel_open, set_panel_open) = signal(true);

    view! {
        <div class=move || {
            format!(
                "absolute left-0 top-0 h-full
                transition-transform duration-300 ease-in-out {}",
                if panel_open.get() { "translate-x-0" } else { "-translate-x-full" },
            )
        }>
            <div class="h-full w-md overflow-y-auto
            bg-neutral-950 border-r border-zinc-800
            p-1 xl:p-3">
                <h2 class="text-shadow-md/50 shadow-gray-950 text-amber-300
                text-sm xl:text-base mb-2 mt-2 font-display
                font-bold leading-none tracking-tight">"Total Effects"</h2>

                <ul class="list-none xl:space-y-1 text-xs xl:text-sm">
                    {move || {
                        let stats = stats.with(|stats| stats.into());
                        effects_tooltip::formatted_effects_list(stats)
                    }}
                </ul>

                {move || {
                    purchased_nodes
                        .with(|purchased_nodes| {
                            passives_tree_specs
                                .read()
                                .nodes
                                .iter()
                                .filter(|(node_id, _)| purchased_nodes.contains(*node_id))
                                .flat_map(|(_, node_specs)| node_specs.triggers.iter())
                                .cloned()
                                .map(|trigger_specs| {
                                    view! {
                                        <div class="relative pb-2 list-none">
                                            <Separator />
                                            {trigger_tooltip::format_trigger(trigger_specs)}
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                }}
            </div>

            <button
                class="absolute top-1/2 right-0 -translate-y-1/2 translate-x-full
                rounded-r-md flex items-center justify-center
                w-6 h-16 font-extrabold shadow-md
                border border-zinc-800
                bg-gradient-to-t from-zinc-900 to-zinc-800 
                hover:bg-gradient-to-tr hover:from-zinc-900 hover:to-neutral-700 
                active:bg-gradient-to-t active:from-zinc-900 active:to-zinc-950 
                "
                on:click=move |_| {
                    set_panel_open.update(|v| *v = !*v);
                }
            >
                {move || if panel_open.get() { "<" } else { ">" }}
            </button>
        </div>
    }
}
