use leptos::html::*;
use leptos::prelude::*;

use crate::components::ui::buttons::MainMenuButton;

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let abandon_quest = move |_| navigate("/", Default::default());

    let audio_ref = NodeRef::<Audio>::new();
    view! {
        <div class="flex justify-between items-center p-2 bg-zinc-800 shadow-md">
            <div class="flex justify-around w-full">
                <div>
                    <p class="text-shadow-md shadow-gray-950 text-amber-200 text-xl">"Gold: 0"</p>
                </div>
                <div>
                    <p class="text-shadow-md shadow-gray-950 text-xl">"Level: 1"</p>
                </div>
            </div>
            <div class="flex space-x-2  w-full">
                <audio
                    src="./assets/musics/ambient1.mp3"
                    node_ref=audio_ref
                    autoplay
                    loop
                    controls
                ></audio>
                <MainMenuButton>"Inventory"</MainMenuButton>
                <MainMenuButton>"Passive Skills"</MainMenuButton>
                <MainMenuButton>"Statistics"</MainMenuButton>
                <MainMenuButton on:click=abandon_quest>"Abandon Quest"</MainMenuButton>
            </div>
        </div>
    }
}
