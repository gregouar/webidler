use leptos::html::*;
use leptos::prelude::*;

use crate::components::ui::buttons::MainMenuButton;

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let abandon_quest = move |_| navigate("/", Default::default());

    view! {
        <div class="flex flex-row items-center space-x-2 p-2 bg-zinc-800 shadow-md">
            <div>
                <p class="text-shadow-md shadow-gray-950 text-amber-200 text-2xl">"Menu"</p>
            </div>
            <MainMenuButton>"Inventory"</MainMenuButton>
            <MainMenuButton>"Passive Skills"</MainMenuButton>
            <MainMenuButton>"Statistics"</MainMenuButton>
            <MainMenuButton on:click=abandon_quest>"Abandon Quest"</MainMenuButton>
        </div>
    }
}
