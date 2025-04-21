use leptos::html::*;
use leptos::prelude::*;

use super::monsters_grid::MonstersGrid;
use super::player_card::PlayerCard;

#[component]
pub fn BattleScene() -> impl IntoView {
    view! {
        <div class="w-full grid grid-cols-3 justify-items-stretch flex items-start gap-4 p-4 ">
            <PlayerCard class:col-span-1 class:justify-self-end />
            <MonstersGrid class:col-span-2 class:justify-self-start />
        </div>
    }
}
