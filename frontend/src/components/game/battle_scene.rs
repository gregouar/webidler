use leptos::html::*;
use leptos::prelude::*;

use crate::assets::img_asset;

use super::monsters_grid::MonstersGrid;
use super::player_card::PlayerCard;
use super::GameContext;

#[component]
pub fn BattleScene() -> impl IntoView {
    let game_context = expect_context::<GameContext>();

    let header_background = move || {
        format!(
            "backgroundImage: url('{}');",
            img_asset(&game_context.world_specs.read().header_background)
        )
    };
    let footer_background = move || {
        format!(
            "backgroundImage: url('{}');",
            img_asset(&game_context.world_specs.read().footer_background)
        )
    };

    view! {
        <div class="w-full grid grid-cols-3 justify-items-stretch flex items-start gap-4 p-4 ">
            <PlayerCard class:col-span-1 class:justify-self-end />

            <div class="shadow-lg rounded-md overflow-hidden  w-full col-span-2 justify-self-start">
                <div
                    class="relative overflow-hidden w-full h-16 bg-center bg-repeat-x flex items-center justify-center"
                    style=header_background
                >
                    <div class="absolute inset-0 bg-gradient-to-r from-transparent via-zinc-950 to-transparent blur-lg"></div>
                    <p class="relative text-shadow-sm shadow-gray-950 text-amber-200 text-2xl font-bold">
                        <span class="[font-variant:small-caps]">
                            {game_context.world_specs.read().name.clone()}
                        </span>
                        {move || {
                            format!(
                                " — Area Level: {}",
                                game_context.world_state.read().area_level,
                            )
                        }}
                    </p>
                </div>
                <MonstersGrid />
                <div class="w-full h-16 bg-center bg-repeat-x" style=footer_background></div>
            </div>

        </div>
    }
}
