use leptos::{html::*, prelude::*};

use crate::components::{
    chat::chat_context::ChatContext,
    events::{EventsContext, Key},
    shared::{
        inventory::InventoryEquipFilter,
        resources::{GemsCounter, GoldCounter, ShardsCounter},
    },
    town::TownContext,
    ui::{buttons::MenuButton, fullscreen::FullscreenButton, wiki::WikiButton},
};

#[component]
pub fn HeaderMenu() -> impl IntoView {
    let town_context: TownContext = expect_context();
    let chat_context: ChatContext = expect_context();
    let events_context: EventsContext = expect_context();

    let gold = Signal::derive(move || town_context.character.read().resource_gold);
    let gems = Signal::derive(move || town_context.character.read().resource_gems);
    let shards = Signal::derive(move || town_context.character.read().resource_shards);

    let navigate_quit = {
        let navigate = leptos_router::hooks::use_navigate();
        move |_| {
            navigate("/user-dashboard", Default::default());
        }
    };

    let disable_panels = Signal::derive(move || town_context.character.read().max_area_level == 0);

    let open_inventory = move || {
        town_context
            .open_inventory
            .set(!town_context.open_inventory.get_untracked());
        town_context.open_ascend.set(false);
        town_context.open_temple.set(false);
        town_context.equip_filter.set(InventoryEquipFilter::Slot);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('i')) {
            open_inventory()
        }
    });

    let open_stash = move || {
        town_context
            .open_stash
            .set(!town_context.open_stash.get_untracked());
        town_context.open_ascend.set(false);
        town_context.open_market.set(false);
        town_context.open_forge.set(false);
        town_context.open_temple.set(false);
        town_context.open_inventory.set(false);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('s')) {
            open_stash()
        }
    });

    let open_market = move || {
        town_context
            .open_market
            .set(!town_context.open_market.get_untracked());
        town_context.open_ascend.set(false);
        town_context.open_forge.set(false);
        town_context.open_temple.set(false);
        town_context.open_inventory.set(false);
        town_context.open_stash.set(false);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('m')) {
            open_market()
        }
    });

    let open_forge = move || {
        town_context
            .open_forge
            .set(!town_context.open_forge.get_untracked());
        town_context.open_market.set(false);
        town_context.open_ascend.set(false);
        town_context.open_temple.set(false);
        town_context.open_inventory.set(false);
        town_context.open_stash.set(false);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('r')) {
            open_forge()
        }
    });

    let open_ascend = move || {
        town_context
            .open_ascend
            .set(!town_context.open_ascend.get_untracked());
        town_context.open_market.set(false);
        town_context.open_forge.set(false);
        town_context.open_temple.set(false);
        town_context.open_inventory.set(false);
        town_context.open_stash.set(false);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('p')) {
            open_ascend()
        }
    });

    let open_temple = move || {
        town_context
            .open_temple
            .set(!town_context.open_temple.get_untracked());
        town_context.open_ascend.set(false);
        town_context.open_market.set(false);
        town_context.open_forge.set(false);
        town_context.open_inventory.set(false);
        town_context.open_stash.set(false);
    };

    Effect::new(move || {
        if events_context.key_pressed(Key::Character('t')) {
            open_temple()
        }
    });

    view! {
        <div class="relative z-50 flex justify-between items-center p-1 xl:p-2
        bg-zinc-800 border-b-1 border-zinc-900/50 shadow-md/30 h-auto">
            <div class="flex justify-around w-full items-center">
                <GoldCounter value=gold w_full=true />
                <GemsCounter value=gems w_full=true />
                <ShardsCounter value=shards w_full=true />
            </div>
            <div class="flex justify-end space-x-1 xl:space-x-2 w-full">
                <FullscreenButton />
                <WikiButton />
                <MenuButton
                    class:hidden
                    class:xl:inline
                    on:click=move |_| {
                        chat_context.opened.set(!chat_context.opened.get_untracked())
                    }
                >
                    "Chat"
                </MenuButton>
                <MenuButton on:click=move |_| open_inventory() disabled=disable_panels>
                    "Inventory"
                </MenuButton>
                <MenuButton on:click=move |_| open_stash() disabled=disable_panels>
                    "Stash"
                </MenuButton>
                <MenuButton on:click=move |_| open_market() disabled=disable_panels>
                    "Market"
                    {move || {
                        (town_context.market_stash.read().resource_gems > 0.0).then_some(" [!]")
                    }}

                </MenuButton>
                <MenuButton on:click=move |_| open_forge() disabled=disable_panels>
                    "Forge"
                </MenuButton>
                <MenuButton on:click=move |_| open_ascend() disabled=disable_panels>
                    "Passives"
                </MenuButton>
                <MenuButton on:click=move |_| open_temple() disabled=disable_panels>
                    "Temple"
                </MenuButton>
                <MenuButton on:click=navigate_quit>"Back"</MenuButton>
            </div>
        </div>
    }
}
