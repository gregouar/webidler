use leptos::html::*;
use leptos::prelude::*;

use crate::assets::music_asset;
use crate::components::ui::buttons::MenuButton;

use super::GameContext;

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let navigate = leptos_router::hooks::use_navigate();
    let abandon_quest = move |_| navigate("/", Default::default());

    // let audio_ref = NodeRef::<Audio>::new();
    let musics: Vec<String> = game_context
        .world_specs
        .read()
        .musics
        .iter()
        .map(|m| music_asset(m))
        .collect();

    let gold = move || game_context.player_state.read().gold;

    view! {
        <div class="relative z-50 flex justify-between items-center p-2 bg-zinc-800 shadow-md h-auto">
            <div class="flex justify-around w-full">
                <div>
                    <p class="text-shadow-md shadow-gray-950 text-amber-200 text-xl">
                        {move || format!("Gold: {:.2e}", gold())}
                    </p>
                </div>
                <div>
                    <p class="text-shadow-md shadow-gray-950 text-xl">"Magic Essence: 0"</p>
                </div>
                <div>
                    <p class="text-shadow-md shadow-gray-950 text-xl">"Gems: 0"</p>
                </div>
                <div>
                    <p class="text-shadow-md shadow-gray-950 text-xl">"Power Shards: 0"</p>
                </div>
            </div>
            <div class="flex space-x-2  w-full">
                // <audio src=musics node_ref=audio_ref autoplay loop controls></audio>
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
