use leptos::{html::*, prelude::*};

use crate::components::{
    game::header_menu::ResourceCounter, town::TownContext, ui::buttons::MenuButton,
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

    let disable_buttons = Signal::derive(|| true);

    view! {
        <div class="relative z-50 w-full flex justify-between items-center p-1 lg:p-2 bg-zinc-800 shadow-md h-auto">
            <div class="flex justify-around w-full items-center">
                <ResourceCounter
                    class:text-violet-300
                    icon="ui/gems.webp"
                    name="Gems"
                    description="To buy items in the market between grinds."
                    value=gems
                />
                <ResourceCounter
                    class:text-cyan-300
                    icon="ui/power_shard.webp"
                    name="Power Shards"
                    description="To permanently increase power of passive skills."
                    value=shards
                />
            </div>
            <div class="flex justify-end space-x-1 lg:space-x-2 w-full">
                <MenuButton on:click=move |_| {
                    town_context.open_market.set(!town_context.open_market.get());
                    town_context.open_ascend.set(false);
                }>"Market"</MenuButton>
                <MenuButton on:click=move |_| {
                    town_context.open_ascend.set(!town_context.open_ascend.get());
                    town_context.open_market.set(false);
                }>"Ascend"</MenuButton>
                <MenuButton on:click=|_| {} disabled=disable_buttons>
                    "Forge"
                </MenuButton>
                <MenuButton on:click=navigate_quit>"Quit"</MenuButton>
            </div>
        </div>
    }
}
