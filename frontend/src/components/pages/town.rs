use codee::string::JsonSerdeCodec;
use leptos::prelude::*;
use leptos_use::storage;

use shared::data::user::{UserCharacter, UserCharacterActivity, UserCharacterId};

use crate::components::{
    backend_client,
    town::{header_menu::HeaderMenu, town_scene::TownScene},
    ui::tooltip::DynamicTooltip,
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
    view! {
        <main class="my-0 mx-auto w-full text-center overflow-x-hidden flex flex-col min-h-screen">
            <DynamicTooltip />

            // <Transition fallback=move || {
            // view! { <p class="text-gray-400">"Loading..."</p> }
            // }>

            <HeaderMenu />
            <div class="relative flex-1">
                <TownScene />
            // <MarketPanel open=town_context.open_market />
            // <AscendPanel open=town_context.open_ascend />
            // <ForgePanel open=town_context.open_forge />
            </div>
        </main>
    }
}
