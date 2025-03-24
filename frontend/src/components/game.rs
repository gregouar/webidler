use super::progress_bar::{CircularProgressBar, HorizontalProgressBar, VerticalProgressBar};
use leptos::html::*;
use leptos::prelude::*;

use super::buttons::MainMenuButton;

#[component]
pub fn Game() -> impl IntoView {
    view! {
        <main class="my-0 mx-auto text-center text-white font-serif">
            <h1 class="text-shadow-lg shadow-gray-950 mb-4 text-amber-200 text-4xl  md:text-5xl lg:text-6xl font-extrabold leading-none tracking-tight">"Welcome to Webidler!"</h1>
            <div class="grid grid-cols-2 gap-4">
                <AdventurerPanel />
                <MonsterPanel />
            </div>
        </main>
    }
}

#[component]
pub fn AdventurerPanel() -> impl IntoView {
    let (action_bar, set_action_bar) = signal(69.0);
    let (health_bar, set_health_bar) = signal(42.0);
    view! {
        <div class="max-w-lg grid grid-cols-2 grid-rows-2">
            <img src="/assets/adventurers/human_male_2.webp" alt="adventurer" class="border-8 border-double border-stone-500" />
            <VerticalProgressBar class:w-6=true bar_color="bg-red-500" value=health_bar />
            <HorizontalProgressBar  class:h-3=true bar_color="bg-amber-700" value=action_bar />
            <CircularProgressBar  bar_color="text-amber-700" value=action_bar />
            <MainMenuButton on:click=move |_| set_action_bar.set(90.0)>"Attack"</MainMenuButton>
            <MainMenuButton on:click=move |_| set_health_bar.set(100.0)>"Potion"</MainMenuButton>
        </div>
    }
}

#[component]
pub fn MonsterPanel() -> impl IntoView {
    let (action_bar, set_action_bar) = signal(42.0);
    let (health_bar, set_health_bar) = signal(69.0);
    view! {
        <div  class="max-w-lg grid grid-cols-2 grid-rows-2">
            <img src="/assets/monsters/bat2.webp" alt="bat monster3"  class="border-8 border-double border-stone-500"/>
            <VerticalProgressBar class:place-self-start=true class:w-6=true bar_color="bg-red-500" value=health_bar />
            <HorizontalProgressBar class:h-3=true bar_color="bg-amber-700" value=action_bar />
        </div>
    }
}
