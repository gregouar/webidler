use leptos::{html::*, prelude::*};

use crate::components::{
    game::resources::{GemsCounter, ShardsCounter},
    town::TownContext,
    ui::buttons::MenuButton,
};

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let gems = Signal::derive(move || town_context.character.read().resource_gems);
    let shards = Signal::derive(move || town_context.character.read().resource_shards);

    let navigate_quit = {
        let navigate = leptos_router::hooks::use_navigate();
        move |_| {
            navigate("/user-dashboard", Default::default());
        }
    };

    view! {
        <div class="relative z-50 w-full flex justify-between items-center p-1 xl:p-2 bg-zinc-800 shadow-md h-auto">
            <div class="flex justify-around w-full items-center">
                <GemsCounter value=gems />
                <ShardsCounter value=shards />
            </div>
            <div class="flex justify-end space-x-1 xl:space-x-2 w-full">
                <MenuButton on:click=move |_| {
                    town_context.open_market.set(!town_context.open_market.get());
                    town_context.open_ascend.set(false);
                    town_context.open_forge.set(false);
                }>"Market"</MenuButton>
                <MenuButton on:click=move |_| {
                    town_context.open_ascend.set(!town_context.open_ascend.get());
                    town_context.open_market.set(false);
                    town_context.open_forge.set(false);
                }>"Ascend"</MenuButton>
                <MenuButton on:click=move |_| {
                    town_context.open_forge.set(!town_context.open_forge.get());
                    town_context.open_market.set(false);
                    town_context.open_ascend.set(false);
                }>"Forge"</MenuButton>
                <MenuButton on:click=navigate_quit>"Quit"</MenuButton>
            </div>
        </div>
    }
}
