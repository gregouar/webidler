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
        settings::{GraphicsQuality, SettingsContext},
        shared::tooltips::{
            effects_tooltip::{self, formatted_effects_list},
            frame::{TooltipFrame, TooltipFramePalette},
            trigger_tooltip::{self, format_trigger},
        },
        ui::{
            Separator,
            card::CardInsetTitle,
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
    let settings: SettingsContext = expect_context();
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
        if settings.graphics_quality() == GraphicsQuality::Low {
            return "";
        }
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
                                        class=move || {
                                            format!(
                                                "group-active:scale-90 group-active:brightness-100 {}",
                                                if settings.graphics_quality() == GraphicsQuality::High {
                                                    "xl:drop-shadow-[2px_2px_2px_black]"
                                                } else {
                                                    ""
                                                },
                                            )
                                        }
                                        // xl:drop-shadow-[2px_2px_2px_black]
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
    let palette = TooltipFramePalette {
        border_class: "border-teal-700/90",
        inner_border_class: "border-teal-200/10",
        shine_color: "rgba(153,244,234,0.35)",
    };

    view! {
        <TooltipFrame palette class="max-w-xs">
            <NodeTooltipContent node_specs node_level show_upgrade />
        </TooltipFrame>
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
        .map(|trigger| format_trigger(trigger, false))
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
    let settings: SettingsContext = expect_context();
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
            <div class=move || {
                format!(
                    "h-full w-md overflow-y-auto p-2 xl:p-3 border-r {} {} {}",
                    match settings.graphics_quality() {
                        GraphicsQuality::High => "border-[#5a4a30]/70",
                        GraphicsQuality::Medium => "border-[#5a4a30]/70",
                        GraphicsQuality::Low => "border-[#54462f]/80",
                    },
                    match settings.graphics_quality() {
                        GraphicsQuality::High => {
                            "bg-[linear-gradient(180deg,rgba(226,193,122,0.04),rgba(0,0,0,0.02)_24%,rgba(0,0,0,0.14)_100%),linear-gradient(180deg,rgba(19,19,23,0.98),rgba(10,10,12,1))]"
                        }
                        GraphicsQuality::Medium => {
                            "bg-[linear-gradient(180deg,rgba(204,172,105,0.035),rgba(0,0,0,0.02)_24%,rgba(0,0,0,0.12)_100%),linear-gradient(180deg,rgba(19,19,23,0.98),rgba(10,10,12,1))]"
                        }
                        GraphicsQuality::Low => {
                            "bg-[linear-gradient(180deg,rgba(174,145,88,0.03),rgba(0,0,0,0.03)_26%,rgba(0,0,0,0.1)_100%),linear-gradient(180deg,rgba(20,20,24,0.98),rgba(11,11,13,1))]"
                        }
                    },
                    if settings.graphics_quality() == GraphicsQuality::High {
                        "shadow-[inset_0_1px_0_rgba(255,255,255,0.03),inset_-1px_0_0_rgba(0,0,0,0.35)]"
                    } else {
                        ""
                    },
                )
            }>
                <Show when=move || settings.graphics_quality() != GraphicsQuality::Low>
                    <div class="pointer-events-none absolute inset-[1px] border-r border-white/5"></div>
                </Show>

                <CardInsetTitle>"Total Effects"</CardInsetTitle>

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
                                        <div class="pb-2 list-none">
                                            <Separator />
                                            {trigger_tooltip::format_trigger(trigger_specs, true)}
                                        </div>
                                    }
                                })
                                .collect::<Vec<_>>()
                        })
                }}
            </div>

            <button
                class=move || {
                    format!(
                        "absolute top-1/2 right-0 -translate-y-1/2 translate-x-full
                        rounded-r-[7px] flex items-center justify-center
                        w-7 h-18 font-extrabold text-stone-200 text-shadow shadow-black/80
                        border border-l-0 active:brightness-90 {} {}",
                        match settings.graphics_quality() {
                            GraphicsQuality::High => "border-[#5a4a30]/75",
                            GraphicsQuality::Medium => "border-[#5a4a30]/75",
                            GraphicsQuality::Low => "border-[#54462f]/80",
                        },
                        match settings.graphics_quality() {
                            GraphicsQuality::High => {
                                "bg-[linear-gradient(180deg,rgba(214,177,102,0.08),rgba(0,0,0,0.14)),linear-gradient(180deg,rgba(39,38,44,0.98),rgba(18,18,22,1))] shadow-[0_4px_10px_rgba(0,0,0,0.28),inset_0_1px_0_rgba(236,210,148,0.12),inset_0_-1px_0_rgba(0,0,0,0.35)] hover:text-[#f1e4c4] hover:border-[#7b6440]"
                            }
                            GraphicsQuality::Medium => {
                                "bg-[linear-gradient(180deg,rgba(199,166,101,0.06),rgba(0,0,0,0.12)),linear-gradient(180deg,rgba(39,38,44,0.98),rgba(18,18,22,1))] hover:text-[#f1e4c4] hover:border-[#7b6440]"
                            }
                            GraphicsQuality::Low => {
                                "bg-[linear-gradient(180deg,rgba(174,145,88,0.05),rgba(0,0,0,0.1)),linear-gradient(180deg,rgba(37,36,42,0.99),rgba(18,18,22,1))] hover:text-[#e8d8b0] hover:border-[#6d5a3c]"
                            }
                        },
                    )
                }
                on:click=move |_| {
                    set_panel_open.update(|v| *v = !*v);
                }
            >
                {move || if panel_open.get() { "<" } else { ">" }}
            </button>
        </div>
    }
}
