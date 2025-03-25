use super::progress_bar::{CircularProgressBar, HorizontalProgressBar, VerticalProgressBar};
use leptos::html::*;
use leptos::prelude::*;

use super::buttons::MainMenuButton;
use super::icons::attack_icon::AttackIcon;

#[component]
pub fn Game() -> impl IntoView {
    view! {
        <main class="my-0 mx-auto text-center text-white font-serif">
            <h1 class="text-shadow-lg shadow-gray-950 mb-4 text-amber-200 text-4xl  md:text-5xl lg:text-6xl font-extrabold leading-none tracking-tight">"Welcome to Webidler!"</h1>
            <div class="grid grid-cols-2 gap-4 m-4 flex items-start">
                <AdventurerPanel class:justify-self-end/>
                <MonstersPanel class:justify-self-start />
            </div>
        </main>
    }
}

#[component]
pub fn AdventurerPanel() -> impl IntoView {
    let (action_bar, set_action_bar) = signal(69.0);
    let (health_bar, set_health_bar) = signal(42.0);
    view! {
        <div class="max-w-lg grid grid-cols-4 grid-rows-4 gap-2 p-2  bg-zinc-800">
            <div class="col-span-3 row-span-3">
                <img src="/assets/adventurers/human_male_2.webp" alt="adventurer" class="border-8 border-double border-stone-500" />
            </div>
            <div class="row-span-3">
                <VerticalProgressBar class:drop-shadow-lg class:w-6 bar_color="bg-gradient-to-b from-red-500 to-red-700" value=health_bar />
            </div>
            <CircularProgressBar class:drop-shadow-lg  bar_width=4 bar_color="text-amber-700" value=action_bar>
                <AttackIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
            </CircularProgressBar>
            <MainMenuButton on:click=move |_| set_action_bar.set(90.0)>"Attack"</MainMenuButton>
            <MainMenuButton on:click=move |_| set_health_bar.set(100.0)>"Potion"</MainMenuButton>
        </div>
    }
}

#[component]
pub fn MonstersPanel() -> impl IntoView {
    view! {
        <div class=" grid grid-cols-2 gap-4 p-4">
            <MonsterPanel />
            <MonsterPanel />
            <MonsterPanel />
            <MonsterPanel />
            <MonsterPanel />
            <MonsterPanel />
        </div>
    }
}

#[component]
pub fn MonsterPanel() -> impl IntoView {
    let (action_bar, set_action_bar) = signal(42.0);
    let (health_bar, set_health_bar) = signal(69.0);
    view! {
        <div  class="max-w-lg grid grid-cols-2 grid-rows-2  bg-zinc-800 gap-2  p-2">
            <img src="/assets/monsters/bat2.webp" alt="bat monster3"  class="border-8 border-double border-stone-500"/>
            <VerticalProgressBar class:w-6 bar_color="bg-red-500" value=health_bar />
            <HorizontalProgressBar class:h-3 bar_color="bg-amber-700" value=action_bar />
        </div>
    }
}
