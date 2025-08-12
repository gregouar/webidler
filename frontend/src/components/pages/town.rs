use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::storage;

use shared::data::user::{UserCharacter, UserCharacterActivity, UserCharacterId};

use crate::{
    assets::img_asset,
    components::{backend_client, ui::buttons::MenuButton},
};

#[component]
pub fn TownPage() -> impl IntoView {
    let (get_character_id_storage, _, _) =
        storage::use_session_storage::<UserCharacterId, JsonSerdeCodec>("character_id");

    let character = UserCharacter {
        character_id: get_character_id_storage.get_untracked(),
        name: "Poudoune".to_string(),
        portrait: "adventurers/human_female_1.webp".to_string(),
        max_area_level: 172,
        activity: UserCharacterActivity::Rusting,
        resource_gems: 42.0,
        resource_shards: 69.0,
    };
    let grinding_areas = vec![GrindingArea {
        name: "Inn Basement".to_string(),
        min_level: 0,
        max_reached: 172,
        header_img: "worlds/inn_header.webp".to_string(),
        footer_img: "worlds/inn_footer.webp".to_string(),
    }];
    view! {
        <main class="my-0 mx-auto w-full max-w-6xl px-4 sm:px-8 text-center overflow-x-hidden flex flex-col min-h-screen">

            <div class="flex justify-between items-center bg-zinc-900 border-b border-zinc-700 px-4 py-3 rounded-b-lg shadow-lg mb-6">
                <div class="flex items-center gap-4">
                    <img
                        src=img_asset(&character.portrait)
                        class="w-16 h-16 rounded-lg border border-amber-300 shadow"
                    />
                    <div class="text-left">
                        <div class="text-lg font-bold text-amber-200">{character.name.clone()}</div>
                        <div class="flex gap-4 text-sm text-gray-400">
                            <span>"💎" {character.resource_gems}</span>
                            <span>"🔮" {character.resource_shards}</span>
                        </div>
                    </div>
                </div>
                <div class="flex gap-3">
                    <MenuButton on:click=move |_| {}>"Back"</MenuButton>
                    <MenuButton on:click=move |_| {}>"Market"</MenuButton>
                    <MenuButton on:click=move |_| {}>"Forge"</MenuButton>
                    <MenuButton on:click=move |_| {}>"Ascend"</MenuButton>
                </div>
            </div>

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
        </main>
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
