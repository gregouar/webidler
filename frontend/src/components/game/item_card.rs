use std::sync::Arc;

use leptos::{html::*, prelude::*};

use shared::data::item::{ItemRarity, ItemSpecs};

use crate::assets::img_asset;
use crate::components::ui::tooltip::{DynamicTooltipContext, DynamicTooltipPosition};

use super::tooltips::ItemTooltip;

#[component]
pub fn ItemCard(
    item_specs: Arc<ItemSpecs>,
    #[prop(default=DynamicTooltipPosition::Auto)] tooltip_position: DynamicTooltipPosition,
) -> impl IntoView {
    let (border_color, ring_color, shadow_color, gradient) = match item_specs.rarity {
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
        ItemRarity::Unique => (
            "border-amber-700/70",
            "ring-amber-600/30",
            "shadow-amber-700/30",
            "from-amber-900/80 to-gray-950",
        ),
    };

    let icon_asset = img_asset(&item_specs.base.icon);

    let tooltip_context = expect_context::<DynamicTooltipContext>();

    let show_tooltip = move |_| {
        let item_specs = item_specs.clone();
        tooltip_context.set_content(
            move || {
                let item_specs = item_specs.clone();
                view! { <ItemTooltip item_specs=item_specs /> }.into_any()
            },
            tooltip_position,
        );
    };

    let hide_tooltip = {
        let tooltip_context = expect_context::<DynamicTooltipContext>();
        move |_| tooltip_context.hide()
    };

    view! {
        <div
            class=format!(
                "relative group flex items-center justify-center w-full aspect-[2/3]
                rounded-md p-1 bg-gradient-to-br {} border-4 {} ring-2 {} shadow-md {}
                ",
                gradient,
                border_color,
                ring_color,
                shadow_color,
            )
            on:mouseenter=show_tooltip
            on:mouseleave=hide_tooltip
        >
            <img
                src=icon_asset
                class="object-contain max-w-full max-h-full transition-all duration-50 ease-in-out
                group-hover:scale-105 group-hover:brightness-110
                group-active:scale-90 group-active:brightness-90
                "
            />
        </div>
    }
}
