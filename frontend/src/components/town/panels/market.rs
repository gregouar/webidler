use chrono::Utc;
use leptos::{prelude::*, task::spawn_local};
use std::sync::Arc;
use strum::IntoEnumIterator;

use shared::{
    data::{
        item::{ItemCategory, ItemRarity},
        market::{MarketFilters, MarketItem, MarketOrderBy},
        passive::StatEffect,
        skill::{DamageType, SkillType},
        stat_effect::{Modifier, StatType},
        trigger::HitTrigger,
    },
    http::client::{
        BrowseMarketItemsRequest, BuyMarketItemRequest, EditMarketItemRequest,
        RejectMarketItemRequest, SellMarketItemRequest,
    },
    types::{ItemPrice, PaginationLimit, Username},
};

use crate::{
    assets::img_asset,
    components::{
        auth::AuthContext,
        backend_client::BackendClient,
        game::{
            panels::inventory::loot_filter_category_to_str,
            tooltips::effects_tooltip::{format_flat_stat, format_multiplier_stat_name},
        },
        town::{
            items_browser::{ItemDetails, ItemsBrowser, SelectedItem, SelectedMarketItem},
            TownContext,
        },
        ui::{
            buttons::{CloseButton, MenuButton, MenuButtonRed, TabButton},
            dropdown::{DropdownMenu, SearchableDropdownMenu},
            input::{Input, ValidatedInput},
            menu_panel::{MenuPanel, PanelTitle},
            number::format_datetime,
            toast::*,
        },
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum MarketTab {
    Filters,
    Buy,
    Sell,
    Listings,
}

#[component]
pub fn MarketPanel(open: RwSignal<bool>) -> impl IntoView {
    let active_tab = RwSignal::new(MarketTab::Buy);
    let selected_item = RwSignal::new(SelectedItem::None);

    let switch_tab = move |new_tab| {
        selected_item.set(SelectedItem::None);
        active_tab.set(new_tab);
    };

    let town_context: TownContext = expect_context();

    let filters = RwSignal::new(MarketFilters {
        item_level: Some(town_context.character.read_untracked().max_area_level),
        // price: ItemPrice::try_new(town_context.character.read_untracked().resource_gems).ok(),
        ..Default::default()
    });

    view! {
        <MenuPanel open=open>
            <div class="w-full">
                <div class="bg-zinc-800 rounded-md p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col">
                    <div class="px-4 relative z-10 flex items-center justify-between">
                        <PanelTitle>"Market"</PanelTitle>

                        <div class="flex-1 flex justify-center ml-2 xl:ml-4 gap-2 xl:gap-4 w-full max-w-md mx-auto">
                            <TabButton
                                is_active=Signal::derive(move || {
                                    active_tab.get() == MarketTab::Filters
                                })
                                on:click=move |_| { switch_tab(MarketTab::Filters) }
                            >
                                "Filters"
                            </TabButton>
                            <TabButton
                                is_active=Signal::derive(move || active_tab.get() == MarketTab::Buy)
                                on:click=move |_| { switch_tab(MarketTab::Buy) }
                            >
                                "Buy"
                            </TabButton>
                            <TabButton
                                is_active=Signal::derive(move || {
                                    active_tab.get() == MarketTab::Sell
                                })
                                on:click=move |_| { switch_tab(MarketTab::Sell) }
                            >
                                "Sell"
                            </TabButton>
                            <TabButton
                                is_active=Signal::derive(move || {
                                    active_tab.get() == MarketTab::Listings
                                })
                                on:click=move |_| { switch_tab(MarketTab::Listings) }
                            >
                                "Listings"
                            </TabButton>
                        </div>

                        <div class="flex-1"></div>

                        <div class="flex items-center gap-2 mb-2">
                            <CloseButton on:click=move |_| open.set(false) />
                        </div>
                    </div>

                    <div class="grid grid-cols-2 gap-2">
                        <div class="w-full aspect-[4/3] bg-neutral-900 overflow-y-auto ring-1 ring-neutral-950 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                            {move || {
                                match active_tab.get() {
                                    MarketTab::Filters => {
                                        view! { <MainFilters filters /> }.into_any()
                                    }
                                    MarketTab::Buy => {
                                        view! {
                                            <MarketBrowser selected_item filters own_listings=false />
                                        }
                                            .into_any()
                                    }
                                    MarketTab::Sell => {
                                        view! { <InventoryBrowser selected_item /> }.into_any()
                                    }
                                    MarketTab::Listings => {
                                        view! {
                                            <MarketBrowser selected_item filters own_listings=true />
                                        }
                                            .into_any()
                                    }
                                }
                            }}
                        </div>

                        <div class="w-full aspect-[4/3] bg-neutral-900 overflow-y-auto shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                            {move || {
                                match active_tab.get() {
                                    MarketTab::Filters => {
                                        view! { <StatsFilters filters /> }.into_any()
                                    }
                                    MarketTab::Buy => {
                                        view! { <BuyDetails selected_item /> }.into_any()
                                    }
                                    MarketTab::Sell => {
                                        view! { <SellDetails selected_item /> }.into_any()
                                    }
                                    MarketTab::Listings => {
                                        view! { <ListingDetails selected_item /> }.into_any()
                                    }
                                }
                            }}
                        </div>
                    </div>

                    <div class="px-4 relative z-10 flex items-center justify-between"></div>
                </div>
            </div>
        </MenuPanel>
    }
}

impl From<MarketItem> for SelectedMarketItem {
    fn from(value: MarketItem) -> Self {
        Self {
            index: value.item_id,
            item_specs: Arc::new(value.item_specs),
            price: value.price,
            owner_id: value.owner_id,
            owner_name: value.owner_name,
            recipient: value.recipient,
            rejected: value.rejected,
            created_at: value.created_at,
        }
    }
}

pub fn item_rarity_str(item_rarity: Option<ItemRarity>) -> &'static str {
    match item_rarity {
        None => "Any",
        Some(ItemRarity::Normal) => "Common",
        Some(ItemRarity::Magic) => "Magical",
        Some(ItemRarity::Rare) => "Rare",
        Some(ItemRarity::Unique) => "Unique",
    }
}

#[component]
fn MarketBrowser(
    selected_item: RwSignal<SelectedItem>,
    filters: RwSignal<MarketFilters>,
    own_listings: bool,
) -> impl IntoView {
    let items_per_page = PaginationLimit::try_new(10).unwrap_or_default();

    let items_list = RwSignal::new(Vec::new());

    let extend_list = RwSignal::new(0u32);
    let reached_end_of_list = RwSignal::new(false);
    let has_more = RwSignal::new(true);

    let refresh_list = move || {
        items_list.write().drain(..);
        extend_list.set(0);
    };

    Effect::new(move || {
        let _ = filters.read();
        refresh_list()
    });

    Effect::new(move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::None => refresh_list(),
            SelectedItem::InMarket(_) => {}
            SelectedItem::Removed(index) => {
                items_list.update(|items_list| {
                    items_list
                        .iter()
                        .position(|item: &SelectedMarketItem| item.index == *index)
                        .map(|i| items_list.remove(i));
                });
            }
        })
    });

    // Effect::new(move || {
    //     if selected_item.read().is_none() {
    //         refresh_list();
    //     }
    // });

    Effect::new(move || {
        if reached_end_of_list.get() && has_more.get_untracked() {
            (*extend_list.write()) += items_per_page.into_inner() as u32;
        }
    });

    Effect::new({
        let backend = expect_context::<BackendClient>();
        let town_context = expect_context::<TownContext>();

        move || {
            let character_id = town_context.character.read().character_id;
            let skip = extend_list.get();
            let filters = filters.get();
            spawn_local(async move {
                let response = backend
                    .browse_market_items(&BrowseMarketItemsRequest {
                        character_id,
                        skip,
                        limit: items_per_page,
                        filters,
                        own_listings,
                    })
                    .await
                    .unwrap_or_default();

                if let Some(mut items_list) = items_list.try_write() {
                    items_list.extend(response.items.into_iter().map(Into::into))
                }
                reached_end_of_list.try_set(false);
                has_more.try_set(response.has_more);
            });
        }
    });
    view! { <ItemsBrowser selected_item items_list reached_end_of_list has_more /> }
}

#[component]
fn InventoryBrowser(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let items_list = Signal::derive({
        let town_context = expect_context::<TownContext>();
        move || {
            town_context
                .inventory
                .read()
                .bag
                .iter()
                .enumerate()
                .map(|(index, item)| SelectedMarketItem {
                    index,
                    owner_id: town_context.character.read_untracked().character_id,
                    owner_name: town_context.character.read_untracked().name.clone(),
                    recipient: None,
                    item_specs: Arc::new(item.clone()),
                    price: 0.0,
                    rejected: false,
                    created_at: Utc::now(),
                })
                .collect::<Vec<_>>()
        }
    });

    view! { <ItemsBrowser selected_item items_list /> }
}

#[component]
pub fn BuyDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let disabled = Signal::derive({
        let town_context = expect_context::<TownContext>();
        move || {
            selected_item.with(|selected_item| match selected_item {
                SelectedItem::InMarket(selected_item) => {
                    selected_item.price > town_context.character.read().resource_gems
                        || selected_item.item_specs.modifiers.level
                            > town_context.character.read().max_area_level
                }
                _ => true,
            })
        }
    });

    let price = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => Some(selected_item.price),
            _ => None,
        })
    };

    let private_offer = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => selected_item.recipient.is_some(),
            _ => false,
        })
    };

    let seller_name = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => selected_item.owner_name.to_string(),
            _ => "".into(),
        })
    };

    let listed_at = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => Some(selected_item.created_at),
            _ => None,
        })
    };

    let do_buy = {
        let character_id = town_context.character.read_untracked().character_id;
        move |_| match selected_item.get() {
            SelectedItem::InMarket(item) => {
                spawn_local({
                    async move {
                        match backend
                            .buy_market_item(
                                &auth_context.token(),
                                &BuyMarketItemRequest {
                                    character_id,
                                    item_index: item.index as u32,
                                },
                            )
                            .await
                        {
                            Ok(response) => {
                                town_context.inventory.set(response.inventory);
                                town_context.character.write().resource_gems =
                                    response.resource_gems;
                                selected_item.set(SelectedItem::Removed(item.index));
                            }
                            Err(e) => show_toast(
                                toaster,
                                format!("Failed to buy item: {e}"),
                                ToastVariant::Error,
                            ),
                        }
                    }
                });
            }
            _ => {}
        }
    };

    let do_reject = {
        let character_id = town_context.character.read_untracked().character_id;
        move |_| match selected_item.get() {
            SelectedItem::InMarket(item) => {
                spawn_local({
                    async move {
                        match backend
                            .reject_market_item(
                                &auth_context.token(),
                                &RejectMarketItemRequest {
                                    character_id,
                                    item_index: item.index as u32,
                                },
                            )
                            .await
                        {
                            Ok(_) => {
                                selected_item.set(SelectedItem::Removed(item.index));
                            }
                            Err(e) => show_toast(
                                toaster,
                                format!("Failed to reject: {e}"),
                                ToastVariant::Error,
                            ),
                        }
                    }
                });
            }
            _ => {}
        }
    };

    view! {
        <div class="w-full h-full flex flex-col justify-between p-4 relative">

            <span class="text-xl font-semibold text-amber-200 text-shadow-md text-center mb-2">
                "Buy from Market"
            </span>

            <div class="flex flex-col">
                <span class="text-pink-400 p-2 font-bold">
                    {move || private_offer().then_some("Private Offer")}
                </span>
                <ItemDetails selected_item />
                <div class="flex justify-between items-center text-sm text-gray-400 p-2">
                    <span>"Listed by: "{move || seller_name()}</span>
                    <span>{move || listed_at().map(format_datetime)}</span>
                </div>
            </div>

            <div class="flex justify-between items-center p-4 border-t border-zinc-700">
                <div class="flex items-center gap-1 text-lg text-gray-400">
                    {move || {
                        price()
                            .map(|price| {
                                if price > 0.0 {
                                    view! {
                                        "Price: "
                                        <span class="text-violet-300 font-bold">
                                            {format!("{:.0}", price)}
                                        </span>
                                        <img
                                            draggable="false"
                                            src=img_asset("ui/gems.webp")
                                            alt="Gems"
                                            class="h-[2em] aspect-square mr-1"
                                        />
                                    }
                                        .into_any()
                                } else {
                                    view! { <span class="text-violet-300 font-bold">"Free"</span> }
                                        .into_any()
                                }
                            })
                    }}
                </div>

                {move || {
                    (private_offer())
                        .then(|| {
                            view! { <MenuButtonRed on:click=do_reject>"Reject"</MenuButtonRed> }
                        })
                }}

                <MenuButton on:click=do_buy disabled=disabled>
                    {move || if price().unwrap_or(1.0) > 0.0 { "Buy Item" } else { "Take Item" }}
                </MenuButton>
            </div>
        </div>
    }
}

#[component]
pub fn SellDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let price = RwSignal::new(None::<ItemPrice>);
    let recipient_name = RwSignal::new(Some(None::<Username>));

    let disabled = Signal::derive(move || {
        selected_item.read().is_empty() || price.read().is_none() || recipient_name.read().is_none()
    });

    let do_sell = {
        let character_id = town_context.character.read_untracked().character_id;
        move |_| match selected_item.get() {
            SelectedItem::InMarket(item) => {
                let recipient_name = recipient_name.get().unwrap_or_default();
                let price = price.get().unwrap().into_inner();
                spawn_local({
                    async move {
                        match backend
                            .sell_market_item(
                                &auth_context.token(),
                                &SellMarketItemRequest {
                                    character_id,
                                    recipient_name,
                                    item_index: item.index,
                                    price,
                                },
                            )
                            .await
                        {
                            Ok(response) => {
                                town_context.inventory.set(response.inventory);
                                selected_item.set(SelectedItem::Removed(item.index));
                            }
                            Err(e) => show_toast(
                                toaster,
                                format!("Failed to list item: {e}"),
                                ToastVariant::Error,
                            ),
                        }
                    }
                });
            }
            _ => {}
        }
    };

    view! {
        <div class="w-full h-full flex flex-col justify-between p-4 relative">
            <span class="text-xl font-semibold text-amber-200 text-shadow-md text-center">
                "Sell from Bag"
            </span>

            <div>
                <ValidatedInput
                    id="private_offer"
                    label="Private Offer:"
                    input_type="text"
                    placeholder="Enter Character Name"
                    bind=recipient_name
                />
            </div>

            <ItemDetails selected_item />

            <div class="flex justify-between items-end p-4 border-t border-zinc-700">
                <div class="flex items-end gap-1 text-lg text-gray-400 ">
                    <ValidatedInput
                        id="price"
                        label="Price:"
                        input_type="number"
                        placeholder="Enter Price"
                        bind=price
                    />
                    <div class="flex items-center">
                        <img
                            draggable="false"
                            src=img_asset("ui/gems.webp")
                            alt="Gems"
                            class="h-[2em] aspect-square mr-1"
                        />
                    </div>
                </div>

                <MenuButton on:click=do_sell disabled=disabled>
                    {move || {
                        if price.get().map(|price| price.into_inner()).unwrap_or(1.0) > 0.0 {
                            "Sell Item"
                        } else {
                            "Give Item"
                        }
                    }}
                </MenuButton>
            </div>
        </div>
    }
}

#[component]
pub fn ListingDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let disabled = Signal::derive(move || selected_item.read().is_empty());

    let price = RwSignal::new(ItemPrice::try_new(0.0).ok());

    Effect::new(move || {
        price.set(
            ItemPrice::try_new(selected_item.with(|selected_item| match selected_item {
                SelectedItem::InMarket(selected_item) => selected_item.price,
                _ => 0.0,
            }))
            .ok(),
        );
    });

    let recipient_name = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => selected_item
                .recipient
                .as_ref()
                .map(|(_, recipient_name)| recipient_name.clone()),
            _ => None,
        })
    };

    let rejected = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => selected_item.rejected,
            _ => false,
        })
    };

    let seller_name = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => selected_item.owner_name.to_string(),
            _ => "".into(),
        })
    };

    let listed_at = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => Some(selected_item.created_at),
            _ => None,
        })
    };

    let do_edit = {
        let character_id = town_context.character.read_untracked().character_id;
        move |_| match selected_item.get() {
            SelectedItem::InMarket(item) => {
                let price = price.get().unwrap().into_inner();
                spawn_local({
                    async move {
                        match backend
                            .edit_market_item(
                                &auth_context.token(),
                                &EditMarketItemRequest {
                                    character_id,
                                    item_index: item.index as u32,
                                    price,
                                },
                            )
                            .await
                        {
                            Ok(_) => {
                                // To trigger full refresh
                                selected_item.set(SelectedItem::None);
                            }
                            Err(e) => show_toast(
                                toaster,
                                format!("Failed to edit listing: {e}"),
                                ToastVariant::Error,
                            ),
                        }
                    }
                });
            }
            _ => {}
        }
    };

    let do_remove = {
        let character_id = town_context.character.read_untracked().character_id;
        move |_| match selected_item.get() {
            SelectedItem::InMarket(item) => {
                spawn_local({
                    async move {
                        match backend
                            .buy_market_item(
                                &auth_context.token(),
                                &BuyMarketItemRequest {
                                    character_id,
                                    item_index: item.index as u32,
                                },
                            )
                            .await
                        {
                            Ok(response) => {
                                town_context.inventory.set(response.inventory);
                                town_context.character.write().resource_gems =
                                    response.resource_gems;
                                selected_item.set(SelectedItem::Removed(item.index));
                            }
                            Err(e) => show_toast(
                                toaster,
                                format!("Failed to remove listing: {e}"),
                                ToastVariant::Error,
                            ),
                        }
                    }
                });
            }
            _ => {}
        }
    };

    view! {
        <div class="w-full h-full flex flex-col justify-between p-4 relative">
            <span class="text-xl font-semibold text-amber-200 text-shadow-md text-center">
                "Remove from Market"
            </span>

            <div class="flex flex-col">
                <span class="p-2">
                    {move || {
                        rejected()
                            .then(|| {
                                view! { <span class="text-red-500 font-bold">"[Rejected] "</span> }
                            })
                    }}
                    {move || {
                        recipient_name()
                            .map(|private_offer| {
                                view! {
                                    <span class="text-pink-400 font-bold">"Private Offer: "</span>
                                    {private_offer}
                                }
                            })
                    }}
                </span>
                <ItemDetails selected_item />
                <div class="flex justify-between items-center text-sm text-gray-400 p-2">
                    <span>"Listed by: "{move || seller_name()}</span>
                    <span>{move || listed_at().map(format_datetime)}</span>
                </div>
            </div>

            <div class="flex justify-between items-end p-4 border-t border-zinc-700">
                <div class="flex items-end gap-1 text-lg text-gray-400 ">
                    <ValidatedInput
                        id="price"
                        label="Price:"
                        input_type="number"
                        placeholder="Enter Price"
                        bind=price
                    />
                    <div class="flex items-center">
                        <img
                            draggable="false"
                            src=img_asset("ui/gems.webp")
                            alt="Gems"
                            class="h-[2em] aspect-square mr-1"
                        />
                    </div>
                    <MenuButton on:click=do_edit disabled=disabled>
                        "Edit Price"
                    </MenuButton>
                </div>

                <MenuButton on:click=do_remove disabled=disabled>
                    "Remove Item"
                </MenuButton>
            </div>
        </div>
    }
}

#[component]
fn MainFilters(filters: RwSignal<MarketFilters>) -> impl IntoView {
    // Inputs

    let item_name = RwSignal::new(Some(filters.get_untracked().item_name));
    Effect::new(move || filters.write().item_name = item_name.get().unwrap_or_default());

    let item_level = RwSignal::new(Some(filters.get_untracked().item_level));
    Effect::new(move || filters.write().item_level = item_level.get().unwrap_or_default());

    let price = RwSignal::new(Some(filters.get_untracked().price));
    Effect::new(move || filters.write().price = price.get().unwrap_or_default());

    let item_damages = RwSignal::new(Some(filters.get_untracked().item_damages));
    Effect::new(move || filters.write().item_damages = item_damages.get().unwrap_or_default());

    let item_armor = RwSignal::new(Some(filters.get_untracked().item_armor));
    Effect::new(move || filters.write().item_armor = item_armor.get().unwrap_or_default());

    let item_block = RwSignal::new(Some(filters.get_untracked().item_block));
    Effect::new(move || filters.write().item_block = item_block.get().unwrap_or_default());

    // Dropdowns

    let item_rarity = RwSignal::new(filters.get_untracked().item_rarity);
    Effect::new(move || filters.write().item_rarity = item_rarity.get());
    let item_rarity_options = std::iter::once(None)
        .chain(ItemRarity::iter().map(Some))
        .map(|rarity| (rarity, item_rarity_str(rarity).into()))
        .collect();

    let item_category = RwSignal::new(filters.get_untracked().item_category);
    Effect::new(move || filters.write().item_category = item_category.get());
    let item_category_options = std::iter::once(None)
        .chain(ItemCategory::iter().map(Some))
        .map(|category| (category, loot_filter_category_to_str(category).into()))
        .collect();

    let order_by = RwSignal::new(filters.get_untracked().order_by);
    Effect::new(move || filters.write().order_by = order_by.get());
    let order_by_options = MarketOrderBy::iter()
        .map(|category| {
            (
                category,
                match category {
                    MarketOrderBy::Price => "Lowest Price",
                    MarketOrderBy::Level => "Lowest Level",
                    MarketOrderBy::Damages => "Highest Damages",
                    MarketOrderBy::Armor => "Highest Armor",
                    MarketOrderBy::Block => "Highest Block Chances",
                }
                .into(),
            )
        })
        .collect();

    view! {
        <div class="w-full h-full flex flex-col p-4 relative">
            <span class="text-xl font-semibold text-amber-200 text-shadow-md text-center mb-2">
                "Main Filters"
            </span>

            <div class="grid grid-cols-1 xl:grid-cols-2 gap-4 p-4 border-b border-zinc-700">
                <div class="flex flex-col gap-4">
                    <ValidatedInput
                        id="item_name"
                        label="Item Name:"
                        input_type="text"
                        placeholder="Enter item name"
                        bind=item_name
                    />

                    <ValidatedInput
                        id="item_level"
                        label="Max Item Level:"
                        input_type="number"
                        placeholder="Enter max item level"
                        bind=item_level
                    />

                    <ValidatedInput
                        id="price"
                        label="Max Price:"
                        input_type="number"
                        placeholder="Enter max price"
                        bind=price
                    />
                </div>

                <div class="flex flex-col gap-4">
                    <div class="flex items-center justify-between text-gray-300 text-sm">
                        <span>"Item Category:"</span>
                        <SearchableDropdownMenu
                            options=item_category_options
                            chosen_option=item_category
                        />
                    </div>

                    <div class="flex items-center justify-between text-gray-300 text-sm">
                        <span>"Item Rarity:"</span>
                        <DropdownMenu options=item_rarity_options chosen_option=item_rarity />
                    </div>

                    <div class="flex items-center justify-between text-gray-300 text-sm">
                        <span>"Order by:"</span>
                        <DropdownMenu options=order_by_options chosen_option=order_by />
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-1 xl:grid-cols-2 gap-4 p-4">
                <div class="flex flex-col gap-4">
                    <ValidatedInput
                        id="item_damages"
                        label="Min Damages:"
                        input_type="number"
                        placeholder="Minimum Damages per second"
                        bind=item_damages
                    />
                    <ValidatedInput
                        id="item_armor"
                        label="Min Armor:"
                        input_type="number"
                        placeholder="Minimum Armor"
                        bind=item_armor
                    />
                    <ValidatedInput
                        id="item_block"
                        label="Min Block %:"
                        input_type="number"
                        placeholder="Minimum Block Percent Chances"
                        bind=item_block
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
fn StatsFilters(filters: RwSignal<MarketFilters>) -> impl IntoView {
    let stat_filters = filters.get_untracked().stat_filters.map(|stat_effect| {
        (
            RwSignal::new(
                stat_effect
                    .as_ref()
                    .map(|stat_effect| (stat_effect.stat, stat_effect.modifier)),
            ),
            RwSignal::new(stat_effect.as_ref().map(|stat_effect| stat_effect.value)),
        )
    });

    Effect::new(move || {
        for (i, (stat_type, stat_value)) in stat_filters.iter().enumerate() {
            filters.write().stat_filters[i] = stat_type.get().map(|(stat, modifier)| StatEffect {
                stat,
                modifier,
                value: stat_value.get().unwrap_or_default(),
            })
        }
    });

    view! {
        <div class="w-full h-full flex flex-col p-4 relative">
            <span class="text-xl font-semibold text-amber-200 text-shadow-md text-center mb-2">
                "Stat Filters"
            </span>

            <div class="flex flex-col gap-2 xl:gap-4 p-2 xl:p-4 border-b border-zinc-700">
                {stat_filters
                    .map(|(stat_type, stat_value)| {
                        view! {
                            <div class="flex gap-2 xl:gap-4 items-center">
                                {move || {
                                    stat_type
                                        .read()
                                        .is_some()
                                        .then(|| {
                                            view! {
                                                <MenuButton
                                                    class:flex-none
                                                    on:click=move |_| {
                                                        stat_type.set(None);
                                                        stat_value.set(None);
                                                    }
                                                >
                                                    "❌"
                                                </MenuButton>
                                            }
                                        })
                                }}
                                <span class=move || {
                                    if stat_type.read().is_none() {
                                        "flex-1 text-center"
                                    } else {
                                        "flex-1"
                                    }
                                }>
                                    <StatDropdown chosen_option=stat_type />
                                </span>
                                {move || {
                                    stat_type
                                        .read()
                                        .is_some()
                                        .then(|| {
                                            view! {
                                                <div class="w-36">
                                                    <Input
                                                        id="stat_value_1"
                                                        input_type="number"
                                                        placeholder="Min"
                                                        bind=stat_value
                                                    />
                                                </div>
                                            }
                                        })
                                }}

                            </div>
                        }
                    })}
            </div>
        </div>
    }
}

#[component]
fn StatDropdown(chosen_option: RwSignal<Option<(StatType, Modifier)>>) -> impl IntoView {
    let available_stats = vec![
        (StatType::Life, Modifier::Multiplier),
        (StatType::Life, Modifier::Flat),
        (StatType::LifeRegen, Modifier::Flat),
        (StatType::Mana, Modifier::Multiplier),
        (StatType::Mana, Modifier::Flat),
        (StatType::ManaRegen, Modifier::Flat),
        (StatType::Armor(DamageType::Fire), Modifier::Flat),
        (StatType::Armor(DamageType::Poison), Modifier::Flat),
        (StatType::Armor(DamageType::Storm), Modifier::Flat),
        (
            StatType::Damage {
                skill_type: None,
                damage_type: None,
            },
            Modifier::Multiplier,
        ),
        (
            StatType::Damage {
                skill_type: Some(SkillType::Attack),
                damage_type: None,
            },
            Modifier::Multiplier,
        ),
        (
            StatType::Damage {
                skill_type: Some(SkillType::Spell),
                damage_type: None,
            },
            Modifier::Multiplier,
        ),
        (
            StatType::Damage {
                skill_type: None,
                damage_type: Some(DamageType::Physical),
            },
            Modifier::Multiplier,
        ),
        (
            StatType::Damage {
                skill_type: None,
                damage_type: Some(DamageType::Fire),
            },
            Modifier::Multiplier,
        ),
        (
            StatType::Damage {
                skill_type: None,
                damage_type: Some(DamageType::Poison),
            },
            Modifier::Multiplier,
        ),
        (
            StatType::Damage {
                skill_type: None,
                damage_type: Some(DamageType::Storm),
            },
            Modifier::Multiplier,
        ),
        (StatType::CritChances(None), Modifier::Multiplier),
        (StatType::CritDamage(None), Modifier::Multiplier),
        (StatType::StatusPower(None), Modifier::Multiplier),
        (StatType::StatusDuration(None), Modifier::Multiplier),
        (StatType::Speed(None), Modifier::Multiplier),
        (
            StatType::Speed(Some(SkillType::Attack)),
            Modifier::Multiplier,
        ),
        (
            StatType::Speed(Some(SkillType::Spell)),
            Modifier::Multiplier,
        ),
        (StatType::MovementSpeed, Modifier::Multiplier),
        (StatType::GoldFind, Modifier::Multiplier),
        (
            StatType::LifeOnHit(HitTrigger {
                skill_type: Some(SkillType::Attack),
                range: None,
                is_crit: None,
                is_blocked: None,
                is_hurt: Some(true),
            }),
            Modifier::Flat,
        ),
        (
            StatType::ManaOnHit(HitTrigger {
                skill_type: Some(SkillType::Attack),
                range: None,
                is_crit: None,
                is_blocked: None,
                is_hurt: Some(true),
            }),
            Modifier::Flat,
        ),
    ];

    let options = available_stats
        .into_iter()
        .map(|(stat_type, modifier)| {
            (
                Some((stat_type, modifier)),
                format_stat_filter(stat_type, modifier),
            )
        })
        .collect();

    view! {
        <SearchableDropdownMenu
            options
            chosen_option
            class:w-full
            missing_text="+ Add Stat Filter"
        />
    }
}

fn format_stat_filter(stat_type: StatType, modifier: Modifier) -> String {
    match modifier {
        Modifier::Multiplier => format!("#% Increased {}", format_multiplier_stat_name(stat_type)),
        Modifier::Flat => format_flat_stat(stat_type, None),
    }
}
