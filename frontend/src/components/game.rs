use super::progress_bar::{CircularProgressBar, HorizontalProgressBar, VerticalProgressBar};
use leptos::html::*;
use leptos::prelude::*;

use super::buttons::MainMenuButton;
use super::icons::attack_icon::AttackIcon;
use super::icons::bite_icon::BiteIcon;

#[component]
pub fn Game() -> impl IntoView {
    view! {
        <main class="my-0 mx-auto text-center text-white font-serif">
            <div class="grid grid-cols-8 justify-items-stretch flex items-start gap-4 m-4 ">
                <SideMenu class:col-span-2 />
                <AdventurerPanel class:col-span-3 class:justify-self-end/>
                <MonstersPanel class:col-span-3 class:justify-self-start/>
            </div>
        </main>
    }
}

#[component]
pub fn SideMenu() -> impl IntoView {
    let navigate = leptos_router::hooks::use_navigate();
    let abandon_quest = move |_| navigate("/webidler", Default::default());

    view! {
        <div class="flex flex-col space-y-2 p-2 bg-zinc-800 rounded-md">
            <div>
                <p class="text-shadow-md shadow-gray-950 text-amber-200 text-2xl">
                    Menu
                </p>
            </div>
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
        <div class="flex flex-col gap-2 p-2 bg-zinc-800 rounded-md h-full">
            <div>
                <p class="text-shadow-md shadow-gray-950 text-amber-200 text-xl">
                    Le Poupou
                </p>
            </div>

            <div class="flex gap-2">
                <VerticalProgressBar class:w-3 class:md:w-6 bar_color="bg-gradient-to-b from-red-500 to-red-700" value=health_bar />
                <div class="flex-1">
                    <img src="/webidler/assets/adventurers/human_male_2.webp" alt="adventurer" class="border-8 border-double border-stone-500" />
                </div>
                <VerticalProgressBar class:w-3 class:md:w-6 bar_color="bg-gradient-to-b from-blue-500 to-blue-700" value=mana_bar />
            </div>

            <div class="grid grid-cols-4 gap-2">
                <CircularProgressBar  bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <AttackIcon  class:drop-shadow-lg  class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
                <CircularProgressBar  bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <AttackIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
                <CircularProgressBar   bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <AttackIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
                <CircularProgressBar  bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <AttackIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
                <MainMenuButton on:click=move |_| set_action_bar.set(10.0)>"Potion1"</MainMenuButton>
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
        <div class="grid grid-cols-2 gap-2 h-full">
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
        <div class="flex w-full bg-zinc-800 rounded-md gap-2 p-2">
            <div class="flex flex-col gap-2">
                <HorizontalProgressBar class:h-2 class:sm:h-4 bar_color="bg-gradient-to-b from-red-500 to-red-700" value=health_bar />
                <div class="flex-1">
                    <img src="/webidler/assets/monsters/bat2.webp" alt="bat monster3"  class="border-8 border-double border-stone-500"/>
                </div>
            </div>
            <div class="flex flex-col justify-evenly w-full">
                <CircularProgressBar  bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <BiteIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
                <CircularProgressBar  bar_width=4 bar_color="text-amber-700" value=action_bar>
                    <BiteIcon class:drop-shadow-lg class:w-full class:h-full class:flex-no-shrink class:fill-current />
                </CircularProgressBar>
            </div>
        </div>
    }
}
