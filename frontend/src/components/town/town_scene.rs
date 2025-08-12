use leptos::prelude::*;

use crate::{assets::img_asset, components::backend_client};

#[component]
pub fn TownScene() -> impl IntoView {
    let grinding_areas = vec![GrindingArea {
        name: "Inn Basement".to_string(),
        min_level: 0,
        max_reached: 172,
        header_img: "worlds/inn_header.webp".to_string(),
        footer_img: "worlds/inn_footer.webp".to_string(),
    }];

    view! {
        // <div class="flex justify-between items-center bg-zinc-900 border-b border-zinc-700 px-4 py-3 rounded-b-lg shadow-lg mb-6">
        // <div class="flex items-center gap-4">
        // <img
        // src=img_asset(&character.portrait)
        // class="w-16 h-16 rounded-lg border border-amber-300 shadow"
        // />
        // <div class="text-left">
        // <div class="text-lg font-bold text-amber-200">{character.name.clone()}</div>
        // <div class="flex gap-4 text-sm text-gray-400">
        // <span>"💎" {character.resource_gems}</span>
        // <span>"🔮" {character.resource_shards}</span>
        // </div>
        // </div>
        // </div>
        // <div class="flex gap-3">
        // <MenuButton on:click=move |_| {}>"Back"</MenuButton>
        // <MenuButton on:click=move |_| {}>"Market"</MenuButton>
        // <MenuButton on:click=move |_| {}>"Forge"</MenuButton>
        // <MenuButton on:click=move |_| {}>"Ascend"</MenuButton>
        // </div>
        // </div>

        <div class="w-full grid grid-cols-3 justify-items-stretch flex items-start gap-4 p-4 ">
            <PlayerCard class:col-span-1 class:justify-self-end />

            <div class="shadow-lg rounded-md overflow-hidden  w-full col-span-2 justify-self-start">

                <div class="bg-zinc-800 rounded-xl border border-zinc-700 shadow-inner px-6 py-6 sm:px-8 sm:py-8 text-left space-y-8">
                    <h2 class="text-2xl font-bold text-white mb-4">"Grinding Areas"</h2>
                    <div class="flex gap-6 overflow-x-auto pb-4">
                        <For
                            each=move || grinding_areas.clone()
                            key=|area| area.name.clone()
                            children=move |area| view! { <GrindingAreaCard area=area.clone() /> }
                        />
                    </div>
                </div>
            </div>

        </div>
    }
}

#[component]
fn PlayerCard() -> impl IntoView {
    view! {
        <div class="
        w-full h-full flex flex-col gap-2 p-2 
        bg-zinc-800 
        ring-1 ring-zinc-950
        rounded-md shadow-md 
        ">
            <div>
                <PlayerName />
            </div>

            <div class="flex flex-col gap-2">
                <div class="flex gap-2">
                    <CharacterPortrait
                        image_uri="adventurers/human_female_2.webp".to_string()
                        character_name="player".to_string()
                    />
                </div>

            </div>
        </div>
    }
}

#[component]
pub fn CharacterPortrait(image_uri: String, character_name: String) -> impl IntoView {
    view! {
        <div class="flex items-center justify-center h-full w-full relative overflow-hidden">

            <div class="border-8 border-double border-stone-500  h-full w-full">
                <div
                    class="h-full w-full"
                    style=format!(
                        "background-image: url('{}');",
                        img_asset("ui/paper_background.webp"),
                    )
                >
                    <img
                        src=img_asset(&image_uri)
                        alt=character_name
                        class="object-cover h-full w-full transition-all duration-[5s]"
                    />

                </div>

            </div>

        </div>
    }
}

#[component]
pub fn PlayerName() -> impl IntoView {
    let player_name = Memo::new(move |_| "Poupou".to_string());

    view! {
        <p class="text-shadow-md shadow-gray-950 text-amber-200 text-xl">
            <span class="font-bold">{player_name}</span>
            {move || format!(" — Max Area Level: {}", 172)}
        </p>
    }
}

#[derive(Clone)]
pub struct GrindingArea {
    pub name: String,
    pub min_level: u32,
    pub max_reached: u32,
    pub header_img: String,
    pub footer_img: String,
}

#[component]
fn GrindingAreaCard(area: GrindingArea) -> impl IntoView {
    view! {
        <div class="bg-neutral-900 rounded-xl border border-zinc-700 shadow-md w-64 flex-shrink-0 overflow-hidden hover:border-amber-400 hover:shadow-lg transition">
            <div class="h-20 w-full relative">
                <img src=img_asset(&area.header_img) class="object-cover w-full h-full" />
                <div class="absolute inset-0 bg-black/30"></div>
            </div>
            <div class="p-4 space-y-2">
                <div class="text-lg font-semibold text-amber-200">{area.name}</div>
                <div class="text-sm text-gray-400">Min Lv: {area.min_level}</div>
                <div class="text-sm text-gray-400">Max Lv Reached: {area.max_reached}</div>
            </div>
            <div class="h-12 w-full relative">
                <img src=img_asset(&area.footer_img) class="object-cover w-full h-full" />
                <div class="absolute inset-0 bg-black/20"></div>
            </div>
        </div>
    }
}
