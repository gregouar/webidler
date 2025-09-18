use leptos::{html::*, prelude::*};

use crate::{
    assets::img_asset,
    components::ui::{
        number::Number,
        tooltip::{StaticTooltip, StaticTooltipPosition},
    },
};

#[component]
pub fn ResourceIcon(
    icon: &'static str,
    name: &'static str,
    description: &'static str,
) -> impl IntoView {
    let tooltip = move || {
        view! {
            <div class="flex flex-col space-y-1 w-[20vw] whitespace-normal">
                <div class="font-semibold text-white">{name}</div>
                <div class="text-sm text-zinc-300">{description}</div>
            </div>
        }
    };
    view! {
        <StaticTooltip tooltip=tooltip position=StaticTooltipPosition::Bottom>
            <img draggable="false" src=img_asset(icon) alt=name class="h-[2em] aspect-square" />
        </StaticTooltip>
    }
}

#[component]
pub fn ResourceCounter(
    icon: &'static str,
    name: &'static str,
    description: &'static str,
    value: Signal<f64>,
) -> impl IntoView {
    view! {
        <div class="flex-1 text-shadow-md shadow-gray-950
        text-sm xl:text-xl 
        flex justify-center items-center space-x-1">
            <div class="font-mono tabular-nums w-[8ch] text-right ">
                <Number value=value />
            </div>
            <ResourceIcon icon name description />
        </div>
    }
}

#[component]
pub fn GoldCounter(value: Signal<f64>) -> impl IntoView {
    view! {
        <ResourceCounter
            class:text-amber-200
            icon="ui/gold.webp"
            name="Gold"
            description="To buy level up for skills. Reset at every grind."
            value
        />
    }
}

#[component]
pub fn GemsIcon() -> impl IntoView {
    view! {
        <ResourceIcon
            icon="ui/gems.webp"
            name="Gems"
            description="To exchange items in the market or craft items at the forge, between grinds."
        />
    }
}
#[component]
pub fn GemsCounter(value: Signal<f64>) -> impl IntoView {
    view! {
        <ResourceCounter
            class:text-fuchsia-300
            icon="ui/gems.webp"
            name="Gems"
            description="To exchange items in the market or craft items at the forge, between grinds."
            value
        />
    }
}

#[component]
pub fn ShardsCounter(value: Signal<f64>) -> impl IntoView {
    view! {
        <ResourceCounter
            class:text-cyan-300
            icon="ui/power_shard.webp"
            name="Power Shards"
            description="To permanently increase power of passive skills, between grinds."
            value
        />
    }
}
