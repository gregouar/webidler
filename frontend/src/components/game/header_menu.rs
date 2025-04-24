use leptos::html::*;
use leptos::prelude::*;

use crate::assets::{img_asset, music_asset};
use crate::components::ui::{buttons::MenuButton, number::Number};

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
        <div class="relative z-50 flex justify-between items-center p-1 bg-zinc-800 shadow-md h-auto">
            <div class="flex justify-around w-full items-center">
                <ResourceCounter
                    class:text-amber-200
                    icon="ui/gold.webp"
                    tooltip="Gold"
                    value=gold
                />
                <div class="flex-1">
                    <p class="text-shadow-md shadow-gray-950 text-xl">"Magic Essence: 0"</p>
                </div>
                <ResourceCounter icon="ui/gems.webp" tooltip="Gems" value=gems />
                <div class="flex-1">
                    <p class="text-shadow-md shadow-gray-950 text-xl">"Power Shards: 0"</p>
                </div>
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
                <MenuButton>"Passive Skills"</MenuButton>
                <MenuButton>"Statistics"</MenuButton>
                <MenuButton on:click=abandon_quest>"Abandon Quest"</MenuButton>
            </div>
        </div>
    }
}

#[component]
fn ResourceCounter(icon: &'static str, tooltip: &'static str, value: Signal<f64>) -> impl IntoView {
    view! {
        <div class="flex-1 text-shadow-md shadow-gray-950 text-xl flex justify-center items-center space-x-1">
            <div class="font-mono tabular-nums w-[4ch] text-right">
                <Number value=value />
            </div>
            <img src=img_asset(icon) alt=tooltip class="h-[2em] aspect-square" />
        // TODO: tooltip
        </div>
    }
}
