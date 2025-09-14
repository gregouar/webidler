// TODO
// Add affix price: 1,3,5,10
// Add suffix/prefix: 2,6,10

use chrono::Utc;
use leptos::prelude::*;
use std::sync::Arc;

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    game::resources::GemsIcon,
    town::{
        items_browser::{ItemDetails, ItemsBrowser, SelectedItem, SelectedMarketItem},
        TownContext,
    },
    ui::{
        buttons::{CloseButton, MenuButton},
        menu_panel::{MenuPanel, PanelTitle},
        toast::*,
    },
};

#[component]
pub fn ForgePanel(open: RwSignal<bool>) -> impl IntoView {
    let selected_item = RwSignal::new(SelectedItem::None);

    view! {
        <MenuPanel open=open>
            <div class="w-full">
                <div class="bg-zinc-800 rounded-md p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col">
                    <div class="px-4 relative z-10 flex items-center justify-between">
                        <PanelTitle>"Forge"</PanelTitle>

                        <div class="flex-1"></div>

                        <div class="flex items-center gap-2 mb-2">
                            <CloseButton on:click=move |_| open.set(false) />
                        </div>
                    </div>

                    <div class="grid grid-cols-2 gap-2">
                        <div class="w-full aspect-[4/3] bg-neutral-900 overflow-y-auto ring-1 ring-neutral-950 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                            <InventoryBrowser selected_item />
                        </div>

                        <div class="w-full aspect-[4/3] bg-neutral-900 overflow-y-auto shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                            <ForgeDetails selected_item />
                        </div>
                    </div>

                    <div class="px-4 relative z-10 flex items-center justify-between"></div>
                </div>
            </div>
        </MenuPanel>
    }
}

#[component]
fn InventoryBrowser(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let items_list = Signal::derive({
        let town_context = expect_context::<TownContext>();
        move || {
            town_context.inventory.with(|inventory| {
                inventory
                    .equipped_items()
                    .enumerate()
                    .map(|(index, (_, item))| SelectedMarketItem {
                        index,
                        owner_id: town_context.character.read_untracked().character_id,
                        owner_name: town_context.character.read_untracked().name.clone(),
                        recipient: Some((
                            town_context.character.read_untracked().character_id,
                            "".into(),
                        )),
                        item_specs: Arc::new(*item.clone()),
                        price: 0.0,
                        rejected: false,
                        created_at: Utc::now(),
                    })
                    .chain(inventory.bag.iter().enumerate().map(|(index, item)| {
                        SelectedMarketItem {
                            index: index + 9,
                            owner_id: town_context.character.read_untracked().character_id,
                            owner_name: town_context.character.read_untracked().name.clone(),
                            recipient: None,
                            item_specs: Arc::new(item.clone()),
                            price: 0.0,
                            rejected: false,
                            created_at: Utc::now(),
                        }
                    }))
                    .collect::<Vec<_>>()
            })
        }
    });

    view! { <ItemsBrowser selected_item items_list /> }
}

#[component]
pub fn ForgeDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let add_affix = move |_| {};

    let is_equipped = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => selected_item.recipient.is_some(),
            _ => false,
        })
    };

    let affix_price = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_market_item) => Some(1.0),
            _ => None,
        })
    };

    let prefix_price = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_market_item) => Some(2.0),
            _ => None,
        })
    };

    let suffix_price = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_market_item) => Some(2.0),
            _ => None,
        })
    };

    view! {
        <div class="w-full h-full flex flex-col justify-between p-4 relative">
            <span class="text-xl font-semibold text-amber-200 text-shadow-md text-center">
                "Forge Item"
            </span>

            <div class="flex flex-col">
                <span class="text-pink-400 p-2 font-bold">
                    {move || is_equipped().then_some("Equipped Item")}
                </span>
                <ItemDetails selected_item />
            </div>

            <div class="flex flex-col gap-2">
                <MenuButton
                    on:click=add_affix
                    disabled=Signal::derive({ move || affix_price().is_none() })
                    class:mb-2
                >
                    <div class="w-full flex justify-center items-center gap-1 text-lg text-gray-400">
                        "Add" <span class="text-white font-bold">"Affix"</span>
                        {move || {
                            affix_price()
                                .map(|price| {
                                    view! {
                                        "for "
                                        <span class="text-violet-300 font-bold">{price}</span>
                                        <GemsIcon />
                                    }
                                })
                        }}
                    </div>
                </MenuButton>
                <MenuButton
                    on:click=add_affix
                    disabled=Signal::derive({ move || prefix_price().is_none() })
                >
                    <div class="w-full flex justify-center items-center gap-1 text-lg text-gray-400">
                        "Add" <span class="text-white font-bold">"Prefix"</span>
                        {move || {
                            prefix_price()
                                .map(|price| {
                                    view! {
                                        "for "
                                        <span class="text-violet-300 font-bold">{price}</span>
                                        <GemsIcon />
                                    }
                                })
                        }}
                    </div>
                </MenuButton>
                <MenuButton
                    on:click=add_affix
                    disabled=Signal::derive({ move || suffix_price().is_none() })
                >
                    <div class="w-full flex justify-center items-center gap-1 text-lg text-gray-400">
                        "Add" <span class="text-white font-bold">"Suffix"</span>
                        {move || {
                            suffix_price()
                                .map(|price| {
                                    view! {
                                        "for "
                                        <span class="text-violet-300 font-bold">{price}</span>
                                        <GemsIcon />
                                    }
                                })
                        }}
                    </div>
                </MenuButton>

            </div>
        </div>
    }
}
