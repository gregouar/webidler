use leptos::html::*;
use leptos::prelude::*;

use crate::assets::{img_asset, music_asset};
use crate::components::ui::{
    buttons::MenuButton,
    number::Number,
    tooltip::{StaticTooltip, StaticTooltipPosition},
};

use super::GameContext;

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let navigate = leptos_router::hooks::use_navigate();
    let abandon_quest = move |_| navigate("/", Default::default());

    let musics: Vec<String> = game_context
        .world_specs
        .read()
        .musics
        .iter()
        .map(|m| music_asset(m))
        .collect();

    let gold = Signal::derive(move || game_context.player_resources.read().gold);
    let gems = Signal::derive(move || 0.0);

    view! {
        <div class="relative z-50 flex justify-between items-center p-2 bg-zinc-800 shadow-md h-auto">
            <div class="flex justify-around w-full items-center">
                <ResourceCounter
                    class:text-amber-200
                    icon="ui/gold2.webp"
                    name="Gold"
                    description="To buy level up for skills. Reset at every quest."
                    value=gold
                />
                // TODO: Magic Essence
                // <ResourceCounter icon="ui/magic_essence.webp" tooltip="Magic Essence" value=gems />
                <ResourceCounter
                    icon="ui/gems.webp"
                    name="Gems"
                    description="To buy items in the market between quests."
                    value=gems
                />
                // TODO: Power Shards
                <ResourceCounter
                    icon="ui/power_shard.webp"
                    name="Power Shards"
                    description="To permanently increase power of passive skills."
                    value=gems
                />
            </div>
            <div class="flex space-x-2  w-full">
                <audio autoplay loop controls>
                    {musics
                        .into_iter()
                        .map(|src| {
                            view! { <source src=src /> }
                        })
                        .collect_view()}
                </audio>

                <MenuButton on:click=move |_| {
                    game_context.open_inventory.set(!game_context.open_inventory.get())
                }>"Inventory"</MenuButton>
                <MenuButton>
                    "Passive Skills"
                    {move || {
                        let points = game_context.player_resources.read().passive_points;
                        if points > 0 { format!(" ({})", points) } else { "".to_string() }
                    }}
                </MenuButton>
                <MenuButton>"Statistics"</MenuButton>
                <MenuButton on:click=abandon_quest>"Abandon Quest"</MenuButton>
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
