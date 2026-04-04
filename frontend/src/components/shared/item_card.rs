use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::data::area::AreaLevel;
use shared::data::item::{ItemRarity, ItemSpecs};

use crate::assets::img_asset;
use crate::components::events::{EventsContext, Key};
use crate::components::settings::SettingsContext;
use crate::components::shared::tooltips::item_tooltip::ComparableType;
use crate::components::ui::tooltip::{DynamicTooltipContext, DynamicTooltipPosition};

use super::tooltips::ItemTooltip;

#[component]
pub fn ItemCard(
    item_specs: Arc<ItemSpecs>,
    #[prop(default=None)] comparable_item_specs: Option<Arc<ItemSpecs>>,
    #[prop(default=DynamicTooltipPosition::Auto)] tooltip_position: DynamicTooltipPosition,
    #[prop(default=Signal::derive(|| AreaLevel::MAX))] max_item_level: Signal<AreaLevel>,
) -> impl IntoView {
    let (accent, inner_border, shadow, rarity_wash, rarity_core, edge_glow, frame_shine) =
        match item_specs.modifiers.rarity {
        ItemRarity::Normal => (
            "rgba(126, 112, 82, 0.28)",
            "rgba(214, 219, 229, 0.16)",
            "rgba(0, 0, 0, 0.5)",
            "rgba(210, 215, 224, 0.06)",
            "rgba(255, 255, 255, 0.015)",
            "rgba(214, 219, 229, 0.12)",
            "rgba(230, 230, 236, 0.22)",
        ),
        ItemRarity::Magic => (
            "rgba(126, 112, 82, 0.3)",
            "rgba(182, 219, 255, 0.24)",
            "rgba(23, 44, 89, 0.5)",
            "rgba(75, 126, 235, 0.24)",
            "rgba(48, 86, 196, 0.2)",
            "rgba(118, 178, 255, 0.24)",
            "rgba(144, 205, 255, 0.46)",
        ),
        ItemRarity::Rare => (
            "rgba(126, 112, 82, 0.32)",
            "rgba(255, 232, 160, 0.24)",
            "rgba(67, 47, 11, 0.52)",
            "rgba(173, 124, 26, 0.28)",
            "rgba(108, 76, 8, 0.22)",
            "rgba(245, 208, 112, 0.22)",
            "rgba(255, 226, 145, 0.52)",
        ),
        ItemRarity::Masterwork => (
            "rgba(126, 112, 82, 0.32)",
            "rgba(236, 204, 255, 0.24)",
            "rgba(61, 24, 99, 0.5)",
            "rgba(143, 78, 220, 0.28)",
            "rgba(90, 44, 150, 0.22)",
            "rgba(210, 161, 255, 0.24)",
            "rgba(228, 183, 255, 0.48)",
        ),
        ItemRarity::Unique => (
            "rgba(126, 112, 82, 0.34)",
            "rgba(255, 226, 186, 0.24)",
            "rgba(104, 34, 8, 0.62)",
            "rgba(188, 72, 28, 0.38)",
            "rgba(114, 18, 8, 0.34)",
            "rgba(255, 126, 82, 0.34)",
            "rgba(255, 170, 116, 0.56)",
        ),
    };

    let icon_asset = img_asset(&item_specs.base.icon);
    let is_unique = matches!(item_specs.modifiers.rarity, ItemRarity::Unique);

    let tooltip_context = expect_context::<DynamicTooltipContext>();

    let show_tooltip = move |show_affixes, compare: bool| {
        let item_specs = item_specs.clone();
        let comparable_item_specs = comparable_item_specs.clone();

        tooltip_context.set_content(
            move || {
                let item_specs = item_specs.clone();
                let is_comparable = comparable_item_specs.is_some();
                view! {
                    <div class="flex gap-1 xl:gap-2">
                        {comparable_item_specs
                            .as_ref()
                            .and_then(|comparable_item_specs| {
                                compare
                                    .then(|| {
                                        view! {
                                            <ItemTooltip
                                                item_specs=comparable_item_specs.clone()
                                                show_affixes
                                                comparable=ComparableType::Equipped
                                                max_item_level
                                            />
                                        }
                                    })
                            })}
                        <ItemTooltip
                            item_specs=item_specs
                            show_affixes
                            comparable=if is_comparable {
                                if compare {
                                    ComparableType::Compared
                                } else {
                                    ComparableType::Comparable
                                }
                            } else {
                                ComparableType::NotComparable
                            }
                            max_item_level
                        />

                    </div>
                }
                .into_any()
            },
            tooltip_position,
        );
    };

    let hide_tooltip = {
        let tooltip_context = expect_context::<DynamicTooltipContext>();
        move || tooltip_context.hide()
    };

    // let node_ref = NodeRef::new();
    // let UseMouseInElementReturn { is_outside, .. } = use_mouse_in_element(node_ref);
    // let is_inside = Memo::new(move |_| !is_outside.get());

    let is_inside = RwSignal::new(false);

    let events_context: EventsContext = expect_context();
    let settings_context: SettingsContext = expect_context();

    Effect::new({
        let show_tooltip = show_tooltip.clone();
        let mut tooltip_in_use = false;
        move || {
            if is_inside.get() {
                tooltip_in_use = true;
                show_tooltip(
                    events_context.key_pressed(Key::Alt)
                        || settings_context.read_settings().always_display_affix_tiers,
                    events_context.key_pressed(Key::Ctrl)
                        || settings_context.read_settings().always_compare_items,
                );
            } else if tooltip_in_use {
                tooltip_in_use = false;
                hide_tooltip();
            }
        }
    });

    view! {
        <div
            // node_ref=node_ref
            class="relative group isolate flex items-center justify-center w-full aspect-[2/3] cursor-pointer"

            on:touchstart={
                let show_tooltip = show_tooltip.clone();
                move |_| { show_tooltip(false, false) }
            }
            on:contextmenu=move |ev| {
                ev.prevent_default();
            }
            on:mouseenter=move |_| is_inside.set(true)
            on:mouseleave=move |_| is_inside.set(false)
        >
            <div
                class="pointer-events-none absolute inset-0"
                aria-hidden="true"
                style=format!(
                    "filter: drop-shadow(0 7px 12px {}) drop-shadow(0 2px 3px rgba(0,0,0,0.45));",
                    shadow,
                )
            >
                <div
                    class="absolute inset-0 bg-black/90"
                    style="clip-path: polygon(8px 0, calc(100% - 8px) 0, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 0 calc(100% - 8px), 0 8px);"
                ></div>
            </div>

            <div
                class="absolute inset-0 overflow-hidden border border-[#6c5329]/85 shadow-[inset_0_1px_0_rgba(240,215,159,0.18),inset_0_-1px_0_rgba(0,0,0,0.5)]"
                style=format!(
                    "clip-path: polygon(8px 0, calc(100% - 8px) 0, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 0 calc(100% - 8px), 0 8px);
                    background-image:
                        linear-gradient(180deg, rgba(214,177,102,0.08), rgba(0,0,0,0.2)),
                        radial-gradient(circle at 50% 30%, {}, transparent 68%),
                        linear-gradient(180deg, {}, transparent 42%),
                        linear-gradient(135deg, rgba(46,44,50,0.96), rgba(18,18,22,1));
                    background-blend-mode: screen, screen, screen, normal;",
                    rarity_core,
                    rarity_wash,
                )
            >
                <div
                    class="pointer-events-none absolute inset-[1px]"
                    style=format!(
                        "clip-path: polygon(7px 0, calc(100% - 7px) 0, 100% 7px, 100% calc(100% - 7px), calc(100% - 7px) 100%, 7px 100%, 0 calc(100% - 7px), 0 7px);
                        border: 1px solid {};
                        box-shadow: inset 0 0 0 1px {}, inset 0 0 12px {}, inset 0 16px 18px rgba(255,255,255,0.03), inset 0 -18px 24px rgba(0,0,0,0.32);",
                        accent,
                        inner_border,
                        edge_glow,
                    )
                ></div>
                <span
                    class="pointer-events-none absolute inset-x-[5px] top-[2px] h-[2px]"
                    style=format!(
                        "background: linear-gradient(90deg, transparent, {}, transparent);",
                        frame_shine,
                    )
                ></span>
                <span
                    class="pointer-events-none absolute inset-y-[4px] left-[1px] w-[2px]"
                    style=format!(
                        "background: linear-gradient(180deg, transparent, {}, transparent);",
                        frame_shine,
                    )
                ></span>
                <span
                    class="pointer-events-none absolute inset-y-[4px] right-[1px] w-[2px]"
                    style=format!(
                        "background: linear-gradient(180deg, transparent, {}, transparent);",
                        frame_shine,
                    )
                ></span>
                <span
                    class="pointer-events-none absolute inset-x-[4px] bottom-[1px] h-[2px]"
                    style=if is_unique {
                        "background: linear-gradient(90deg, transparent, rgba(255,197,129,0.88), transparent);"
                    } else {
                        "display:none;"
                    }
                ></span>
                <span class="pointer-events-none absolute inset-x-3 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/45 to-transparent"></span>
                <span
                    class="pointer-events-none absolute left-[2px] top-[2px] bottom-[2px] w-px"
                    style=format!(
                        "background: linear-gradient(180deg, {}, transparent 65%, rgba(0,0,0,0.45));",
                        inner_border,
                    )
                ></span>
            </div>

            <img
                draggable="false"
                src=icon_asset
                class="relative z-10 object-contain max-w-full max-h-full p-1 transition-all duration-75 ease-in-out
                group-hover:scale-[1.045] group-hover:brightness-110
                group-active:scale-[0.96] group-active:brightness-95
                xl:drop-shadow-[0px_6px_6px_black]
                "
            />
        </div>
    }
}
