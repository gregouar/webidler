use leptos::{html::*, prelude::*};

use crate::{
    assets::img_asset,
    components::ui::{
        buttons::MenuButton,
        number::Number,
        tooltip::{StaticTooltip, StaticTooltipPosition},
    },
};

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let gems = Signal::derive(move || 0.0);

    let navigate_quit = {
        let navigate = leptos_router::hooks::use_navigate();
        move |_| {
            navigate("/user-dashboard", Default::default());
        }
    };

    view! {
        <div class="relative z-50 w-full flex justify-between items-center p-2 bg-zinc-800 shadow-md h-auto">
            <div class="flex justify-around w-full items-center">
                <ResourceCounter
                    icon="ui/gems.webp"
                    name="Gems"
                    description="To buy items in the market between grinds."
                    value=gems
                />
                <ResourceCounter
                    icon="ui/power_shard.webp"
                    name="Power Shards"
                    description="To permanently increase power of passive skills."
                    value=gems
                />
            </div>
            <div class="flex justify-end space-x-2  w-full">
                <MenuButton on:click=|_| {}>"Market"</MenuButton>
                <MenuButton on:click=|_| {}>"Ascend"</MenuButton>
                <MenuButton on:click=|_| {}>"Forge"</MenuButton>
                <MenuButton on:click=navigate_quit>"Quit"</MenuButton>
            </div>
        </div>
    }
}

#[component]
fn ResourceCounter(
    icon: &'static str,
    name: &'static str,
    description: &'static str,
    value: Signal<f64>,
) -> impl IntoView {
    let tooltip = move || {
        view! {
            <div class="flex flex-col space-y-1">
                <div class="font-semibold text-white">{name}</div>
                <div class="text-sm text-zinc-300 max-w-xs">{description}</div>
            </div>
        }
    };
    view! {
        <div class="flex-1 text-shadow-md shadow-gray-950 text-xl flex justify-center items-center space-x-1">
            <div class="font-mono tabular-nums w-[8ch] text-right">
                <Number value=value />
            </div>
            <StaticTooltip tooltip=tooltip position=StaticTooltipPosition::Bottom>
                <img src=img_asset(icon) alt=name class="h-[2em] aspect-square" />
            </StaticTooltip>
        </div>
    }
}
