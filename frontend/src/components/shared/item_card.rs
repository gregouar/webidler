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
    max_item_level: Signal<AreaLevel>,
) -> impl IntoView {
    let (border_color, ring_color, shadow_color, gradient) = match item_specs.modifiers.rarity {
        ItemRarity::Normal => (
            "border-gray-600/70",
            "ring-gray-600/20",
            "shadow-gray-800/20",
            "from-gray-900/80 to-gray-950",
        ),
        ItemRarity::Magic => (
            "border-blue-500/70",
            "ring-blue-400/20",
            "shadow-blue-700/20",
            "from-blue-900/80 to-gray-950",
        ),
        ItemRarity::Rare => (
            "border-yellow-400/70",
            "ring-yellow-300/20",
            "shadow-yellow-600/20",
            "from-yellow-900/80 to-gray-950",
        ),
        ItemRarity::Masterwork => (
            "border-fuchsia-400/70",
            "ring-fuchsia-300/20",
            "shadow-fuchsia-600/20",
            "from-fuchsia-900/80 to-gray-950",
        ),
        ItemRarity::Unique => (
            "border-orange-700/70",
            "ring-orange-600/30",
            "shadow-orange-700/30",
            "from-orange-900/80 to-gray-950",
        ),
    };

    let icon_asset = img_asset(&item_specs.base.icon);

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
            class=format!(
                "relative group flex items-center justify-center w-full aspect-[2/3]
                rounded-md p-1 bg-gradient-to-br {} border-4 {} ring-2 {} shadow-md {}
                cursor-pointer
                ",
                gradient,
                border_color,
                ring_color,
                shadow_color,
            )

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

            <img
                draggable="false"
                src=icon_asset
                class="object-contain max-w-full max-h-full transition-all duration-50 ease-in-out
                group-hover:scale-105 group-hover:brightness-110
                group-active:scale-90 group-active:brightness-90
                "
            />
        </div>
    }
}
