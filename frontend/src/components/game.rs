use super::progress_bar::{CircularProgressBar, VerticalProgressBar};
use leptos::html::*;
use leptos::prelude::*;

use super::buttons::MainMenuButton;
use super::icons::attack_icon::AttackIcon;

#[component]
pub fn Game() -> impl IntoView {
    view! {
        <main class="my-0 mx-auto text-center text-white font-serif">
            <h1 class="text-shadow-lg shadow-gray-950 mb-4 text-amber-200 text-4xl md:text-5xl lg:text-6xl font-extrabold leading-none tracking-tight">"Battle Scene"</h1>
            <div class="grid grid-cols-3 justify-items-stretch flex items-start gap-4 m-4 ">
                <SideMenu/>
                <AdventurerPanel class:justify-self-end/>
                <MonstersPanel class:justify-self-start/>
            </div>
        </main>
    }
}

#[component]
pub fn SideMenu() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let abandon_quest = move |_| navigate("/", Default::default());

    view! {
        <div class="max-w-lg flex flex-col space-y-2 p-2 bg-zinc-800">
            <MainMenuButton>
                "Inventory"
            </MainMenuButton>
            <MainMenuButton>
                "Passive Skills"
            </MainMenuButton>
            <MainMenuButton>
                "Statistics"
            </MainMenuButton>
            <MainMenuButton on:click=abandon_quest>
                "Abandon Quest"
            </MainMenuButton>
        </div>
    }
}

#[component]
pub fn AdventurerPanel() -> impl IntoView {
    let (action_bar, set_action_bar) = signal(69.0);
    let (health_bar, set_health_bar) = signal(42.0);
    let (mana_bar, set_mana_bar) = signal(100.0);
    view! {
        <div class="max-w-lg grid grid-rows-5 gap-2 p-2 bg-zinc-800">
            <div class="row-span-3 flex">
                <VerticalProgressBar class:flex-none class:drop-shadow-lg class:w-6 bar_color="bg-gradient-to-b from-red-500 to-red-700" value=health_bar />
                <img class:flex-1 src="/assets/adventurers/human_male_2.webp" alt="adventurer" class="border-8 border-double border-stone-500" />
                <VerticalProgressBar class:flex-none class:drop-shadow-lg class:w-6 bar_color="bg-gradient-to-b from-blue-500 to-blue-700" value=mana_bar />
            </div>
            <div class="grid grid-cols-4 gap-2">
                <CircularProgressBar class:drop-shadow-lg  bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <AttackIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
                <CircularProgressBar class:drop-shadow-lg  bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <AttackIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
                <CircularProgressBar class:drop-shadow-lg  bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <AttackIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
                <CircularProgressBar class:drop-shadow-lg  bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <AttackIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
            </div>
            <div class="grid grid-cols-4 gap-2">
                <MainMenuButton on:click=move |_| set_action_bar.set(90.0)>"Potion1"</MainMenuButton>
                <MainMenuButton on:click=move |_| set_health_bar.set(100.0)>"Potion2"</MainMenuButton>
                <MainMenuButton on:click=move |_| set_mana_bar.set(100.0)>"Potion3"</MainMenuButton>
                <MainMenuButton on:click=move |_| set_mana_bar.set(20.0)>"Trinket"</MainMenuButton>
            </div>
        </div>
    }
}

#[component]
pub fn MonstersPanel() -> impl IntoView {
    view! {
        <div class="grid grid-cols-2 gap-4 p-2">
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
        <div  class="max-w-sm grid grid-cols-4 grid-rows-4 bg-zinc-800 gap-2  p-2">
            <div class="col-span-3 row-span-3">
                <img src="/assets/monsters/bat2.webp" alt="bat monster3"  class="border-8 border-double border-stone-500"/>
            </div>
            <div class="row-span-3">
                <VerticalProgressBar class:w-6 bar_color="bg-gradient-to-b from-red-500 to-red-700" value=health_bar />
            </div>
            <CircularProgressBar class:drop-shadow-lg  bar_width=4 bar_color="text-amber-700" value=action_bar>
                <AttackIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
            </CircularProgressBar>
            <CircularProgressBar class:drop-shadow-lg  bar_width=4 bar_color="text-amber-700" value=action_bar>
                <AttackIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
            </CircularProgressBar>
        </div>
    }
}
