use leptos::{html::*, prelude::*, task::spawn_local};
use leptos_use::{use_infinite_scroll_with_options, UseInfiniteScrollOptions};
use std::sync::Arc;
use strum::IntoEnumIterator;

use shared::{
    data::{
        area::AreaLevel,
        item::{ItemCategory, ItemRarity, ItemSpecs},
        market::MarketItem,
    },
    http::client::{BrowseMarketItemsRequest, BuyMarketItemRequest, SellMarketItemRequest},
    types::{ItemName, ItemPrice, PaginationLimit, Username},
};

use crate::{
    assets::img_asset,
    components::{
        auth::AuthContext,
        backend_client::BackendClient,
        game::{
            item_card::ItemCard, panels::inventory::loot_filter_category_to_str,
            tooltips::item_tooltip::ItemTooltipContent,
        },
        town::TownContext,
        ui::{
            buttons::{CloseButton, MenuButton, TabButton},
            dropdown::DropdownMenu,
            input::ValidatedInput,
            menu_panel::MenuPanel,
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

// TODO: move to share for easy http query
#[derive(Clone)]
pub struct MarketFilters {
    pub item_rarity: Option<ItemRarity>,
}

#[component]
pub fn MarketPanel(open: RwSignal<bool>) -> impl IntoView {
    let active_tab = RwSignal::new(MarketTab::Buy); // TODO: Start on filters?
    let selected_item = RwSignal::new(None::<SelectedItem>);

    let switch_tab = move |new_tab| {
        selected_item.set(None);
        active_tab.set(new_tab);
    };

    view! {
        <MenuPanel open=open>
            <div class="w-full p-4">
                <div class="bg-zinc-800 rounded-md p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col">
                    <div class="px-4 relative z-10 flex items-center justify-between">
                        <span class="flex text-shadow-md shadow-gray-950 text-amber-200 text-xl font-semibold mb-2 mr-6">
                            "Market"
                        </span>

                        <div class="flex-1 flex justify-center gap-4 w-full max-w-md mx-auto">
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
                        <div class="w-full aspect-[4/3] bg-neutral-900 ring-1 ring-neutral-950 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                            {move || {
                                match active_tab.get() {
                                    MarketTab::Filters => view! { <Filters /> }.into_any(),
                                    MarketTab::Buy => {
                                        view! { <BuyBrowser selected_item /> }.into_any()
                                    }
                                    MarketTab::Sell => {
                                        view! { <SellBrowser selected_item /> }.into_any()
                                    }
                                    MarketTab::Listings => {
                                        view! { <ListingsBrowser selected_item /> }.into_any()
                                    }
                                }
                            }}
                        </div>

                        <div class="w-full aspect-[4/3] bg-neutral-900 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                            {move || {
                                match active_tab.get() {
                                    MarketTab::Filters => view! {}.into_any(),
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

#[derive(Clone)]
pub struct SelectedItem {
    pub index: usize,
    pub item_specs: Arc<ItemSpecs>,
    pub price: f64,
}

impl From<MarketItem> for SelectedItem {
    fn from(value: MarketItem) -> Self {
        Self {
            index: value.item_id,
            item_specs: Arc::new(value.item_specs),
            price: value.price,
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
fn Filters() -> impl IntoView {
    let item_name = RwSignal::new(None::<ItemName>);

    // TODO: Default to character max level
    let item_level = RwSignal::new(Some(None::<AreaLevel>));

    let item_rarity = RwSignal::new(None);
    let item_rarity_options = std::iter::once(None)
        .chain(ItemRarity::iter().map(Some))
        .map(|rarity| (rarity, item_rarity_str(rarity).into()))
        .collect();

    let item_category = RwSignal::new(None);
    let item_category_options = std::iter::once(None)
        .chain(ItemCategory::iter().map(Some))
        .map(|category| (category, loot_filter_category_to_str(category).into()))
        .collect();

    let item_price = RwSignal::new(Some(None::<ItemPrice>));

    // TODO: MORE

    view! {
        <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 p-4">
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
                    id="item_price"
                    label="Max Price:"
                    input_type="number"
                    placeholder="Enter max price"
                    bind=item_price
                />
            </div>

            <div class="flex flex-col gap-4">
                <div class="flex items-center justify-between text-gray-300 text-sm">
                    <span>"Item Category:"</span>
                    <DropdownMenu options=item_category_options chosen_option=item_category />
                </div>

                <div class="flex items-center justify-between text-gray-300 text-sm">
                    <span>"Item Rarity:"</span>
                    <DropdownMenu options=item_rarity_options chosen_option=item_rarity />
                </div>
            </div>
        </div>
    }
}

#[component]
fn BuyBrowser(selected_item: RwSignal<Option<SelectedItem>>) -> impl IntoView {
    let items_per_page = PaginationLimit::try_new(6).unwrap_or_default();

    let items_list = RwSignal::new(Vec::new());
    let max_list = RwSignal::new(0);

    let extend_list = RwSignal::new(0u32);
    let reached_end_of_list = RwSignal::new(false);
    let has_more = RwSignal::new(true);

    Effect::new(move || {
        if selected_item.read().is_none() {
            items_list.write().drain(..);
            extend_list.set(0);
        }
    });

    Effect::new(move || {
        if reached_end_of_list.get() && max_list.get_untracked() > items_list.read_untracked().len()
        {
            (*extend_list.write()) += items_per_page.into_inner() as u32;
        }
    });

    Effect::new({
        let backend = expect_context::<BackendClient>();
        let town_context = expect_context::<TownContext>();

        move || {
            let character_id = town_context.character.read().character_id;
            let skip = extend_list.get();
            spawn_local(async move {
                let response = backend
                    .browse_market_items(&BrowseMarketItemsRequest {
                        character_id,
                        skip,
                        limit: items_per_page,
                    })
                    .await
                    .unwrap_or_default();

                max_list.set(response.max_items);
                if let Some(mut items_list) = items_list.try_write() {
                    items_list.extend(response.items.into_iter().map(Into::into))
                }
                reached_end_of_list.try_set(false);

                if items_list
                    .try_read_untracked()
                    .map(|items_list| items_list.len())
                    .unwrap_or_default()
                    >= response.max_items
                {
                    has_more.try_set(false);
                }
            });
        }
    });
    view! { <ItemsBrowser selected_item items_list reached_end_of_list has_more /> }
}

#[component]
fn SellBrowser(selected_item: RwSignal<Option<SelectedItem>>) -> impl IntoView {
    let items_list = Signal::derive({
        let town_context = expect_context::<TownContext>();
        move || {
            town_context
                .inventory
                .read()
                .bag
                .iter()
                .enumerate()
                .map(|(index, item)| SelectedItem {
                    index,
                    item_specs: Arc::new(item.clone()),
                    price: 0.0,
                })
                .collect::<Vec<_>>()
        }
    });

    view! { <ItemsBrowser selected_item items_list /> }
}

#[component]
fn ListingsBrowser(selected_item: RwSignal<Option<SelectedItem>>) -> impl IntoView {
    let items_list = Signal::derive(std::vec::Vec::new);
    view! { <ItemsBrowser selected_item items_list /> }
}

#[component]
pub fn ItemsBrowser(
    selected_item: RwSignal<Option<SelectedItem>>,
    #[prop(into)] items_list: Signal<Vec<SelectedItem>>,
    #[prop(optional)] reached_end_of_list: Option<RwSignal<bool>>,
    #[prop(optional)] has_more: Option<RwSignal<bool>>,
) -> impl IntoView {
    let el = NodeRef::<Div>::new();
    if let Some(reached_end_of_list) = reached_end_of_list {
        use_infinite_scroll_with_options(
            el,
            move |_| async move {
                if !reached_end_of_list.get() {
                    reached_end_of_list.set(true)
                }
            },
            UseInfiniteScrollOptions::default().distance(10.0),
        );
    }
    view! {
        <div node_ref=el class="p-2 gap-2 overflow-y-auto h-full">
            <For
                each=move || items_list.get().into_iter()
                key=|item| item.index
                let:(item)
            >
                <ItemRow
                    item_specs=item.item_specs.clone()
                    on:click=move |_| selected_item.set(Some(item.clone()))
                    price=item.price
                    highlight=move || selected_item.read().as_ref().map(|selected_item| selected_item.index==item.index).unwrap_or_default()
                />
            </For>
            {move || (items_list.read().is_empty() && !has_more.map(|has_more| has_more.get()).unwrap_or_default()).then(|| view!{
                <div class="w-full h-full flex items-center justify-center">
                    <div class="flex flex-col items-center text-center gap-1">
                        <span class="text-gray-400">"No Item Found"</span>
                        <span class="text-gray-400">"Maybe try other filters?"</span>
                    </div>
                </div>
            })}
            {
                move || {
                    reached_end_of_list.and_then(|reached_end_of_list| {
                        (reached_end_of_list.get()
                            && has_more.map(|has_more| has_more.get()).unwrap_or_default())
                        .then(|| view! { <span class="text-gray-400">"Loading..."</span> })
                    })
                }
            }
        </div>
    }
}

#[component]
pub fn ItemRow(
    item_specs: Arc<ItemSpecs>,
    price: f64,
    highlight: impl Fn() -> bool + Send + Sync + 'static,
) -> impl IntoView {
    view! {
        <div class=move || {
            format!(
                "relative flex w-full items-center justify-between p-3 gap-2 cursor-pointer mb-2 shadow-sm transition-colors duration-150 rounded-lg
                bg-neutral-800 hover:bg-neutral-700 {}",
                if highlight() { "ring-2 ring-amber-400" } else { "ring-1 ring-zinc-950" },
            )
        }>
            <div class="relative h-32 aspect-[2/3] flex-shrink-0">
                <ItemCard item_specs=item_specs.clone() />
            </div>

            <div class="flex flex-col w-full">
                <ItemTooltipContent item_specs />
            </div>

            {(price > 0.0)
                .then(|| {
                    view! {
                        <div class="absolute flex bottom-2 right-2 gap-1 items-center ">
                            <span class="text-gray-400">"Price :"</span>
                            <span class="text-violet-300 font-semibold">
                                {format!("{:.0}", price)}
                            </span>
                            <img
                                src=img_asset("ui/gems.webp")
                                alt="Gems"
                                class="h-[2em] aspect-square mr-1"
                            />
                        </div>
                    }
                })}
        </div>
    }
}

#[component]
pub fn BuyDetails(selected_item: RwSignal<Option<SelectedItem>>) -> impl IntoView {
    let disabled = Signal::derive({
        let town_context = expect_context::<TownContext>();
        move || match selected_item.read().as_ref() {
            Some(selected_item) => {
                selected_item.price > town_context.character.read().resource_gems
            }
            None => true,
        }
    });

    let price = move || {
        selected_item
            .read()
            .as_ref()
            .map(|selected_item| selected_item.price)
            .unwrap_or_default()
    };

    let do_buy = {
        let backend = expect_context::<BackendClient>();
        let town_context = expect_context::<TownContext>();
        let auth_context = expect_context::<AuthContext>();
        let toaster = expect_context::<Toasts>();
        let character_id = town_context.character.read_untracked().character_id;
        move |_| {
            if let Some(item) = selected_item.get() {
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
                                selected_item.set(None);
                            }
                            Err(e) => show_toast(
                                toaster,
                                format!("failed to post item for sell: {e}"),
                                ToastVariant::Error,
                            ),
                        }
                    }
                });
            }
        }
    };

    view! {
        <div class="w-full h-full flex flex-col p-4 text-white relative">
            <span class="text-xl font-semibold text-amber-200 text-shadow-md text-center">
                "Buy from Market"
            </span>

            <ItemDetails selected_item />

            <div class="flex justify-between p-4">
                <div class="flex items-center gap-1 text-lg text-gray-400 ">
                    "Price: "
                    <span class="text-violet-300 font-bold">
                        {move || format!("{:.0}", price())}
                    </span>
                    <img
                        src=img_asset("ui/gems.webp")
                        alt="Gems"
                        class="h-[2em] aspect-square mr-1"
                    />
                </div>

                <div>
                    <MenuButton on:click=do_buy disabled=disabled>
                        "Buy Selected Item"
                    </MenuButton>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn SellDetails(selected_item: RwSignal<Option<SelectedItem>>) -> impl IntoView {
    let price = RwSignal::new(None::<ItemPrice>);
    let private_offer = RwSignal::new(Some(None::<Username>));

    let disabled = Signal::derive(move || {
        selected_item.read().is_none() || price.read().is_none() || private_offer.read().is_none()
    });

    let do_sell = {
        let backend = expect_context::<BackendClient>();
        let town_context = expect_context::<TownContext>();
        let auth_context = expect_context::<AuthContext>();
        let toaster = expect_context::<Toasts>();
        let character_id = town_context.character.read_untracked().character_id;
        move |_| {
            if let Some(item) = selected_item.get() {
                let private_offer = private_offer.get().unwrap_or_default();
                let price = price.get().unwrap().into_inner();
                spawn_local({
                    async move {
                        match backend
                            .sell_market_item(
                                &auth_context.token(),
                                &SellMarketItemRequest {
                                    character_id,
                                    private_offer,
                                    item_index: item.index,
                                    price,
                                },
                            )
                            .await
                        {
                            Ok(response) => {
                                town_context.inventory.set(response.inventory);
                                selected_item.set(None);
                            }
                            Err(e) => show_toast(
                                toaster,
                                format!("failed to post item for sell: {e}"),
                                ToastVariant::Error,
                            ),
                        }
                    }
                });
            }
        }
    };

    view! {
        <div class="w-full h-full flex flex-col p-4 text-white relative justify-between">
            <span class="text-xl font-semibold text-amber-200 text-shadow-md text-center">
                "Sell from Bag"
            </span>

            <div>
                <ValidatedInput
                    id="private_offer"
                    label="Private Offer:"
                    input_type="text"
                    placeholder="Enter Character Name"
                    bind=private_offer
                />
            </div>

            <ItemDetails selected_item />

            <div class="flex justify-between">
                <div class="flex items-center gap-1 text-lg text-gray-400 ">
                    // <span class="text-violet-300 font-bold">{format!("{:.0}", price)}</span>
                    <ValidatedInput
                        id="price"
                        label="Price:"
                        input_type="number"
                        placeholder="Enter Price"
                        bind=price
                    />
                    <img
                        src=img_asset("ui/gems.webp")
                        alt="Gems"
                        class="h-[2em] aspect-square mr-1"
                    />
                </div>

                <MenuButton on:click=do_sell disabled=disabled>
                    "Sell Selected Item"
                </MenuButton>
            </div>
        </div>
    }
}

#[component]
pub fn ListingDetails(selected_item: RwSignal<Option<SelectedItem>>) -> impl IntoView {
    let disabled = Signal::derive(move || selected_item.read().is_none());
    let price = RwSignal::new(
        selected_item
            .read_untracked()
            .as_ref()
            .and_then(|selected_item| ItemPrice::try_new(selected_item.price).ok()),
    );

    view! {
        <div class="w-full h-full flex flex-col p-4 text-white relative">
            <span class="text-xl font-semibold text-amber-200 text-shadow-md text-center">
                "Remove from Market"
            </span>

            <ItemDetails selected_item />

            <div class="flex justify-between p-4">
                <div class="flex items-center gap-1 text-lg text-gray-400 ">
                    <ValidatedInput
                        id="price"
                        label="Price:"
                        input_type="number"
                        placeholder="Enter Price"
                        bind=price
                    />
                    <img
                        src=img_asset("ui/gems.webp")
                        alt="Gems"
                        class="h-[2em] aspect-square mr-1"
                    />
                    <MenuButton on:click=move |_| {} disabled=disabled>
                        "Edit Price"
                    </MenuButton>
                </div>

                <div>
                    <MenuButton on:click=move |_| {} disabled=disabled>
                        "Remove Selected Item"
                    </MenuButton>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn ItemDetails(selected_item: RwSignal<Option<SelectedItem>>) -> impl IntoView {
    let item_details = move || {
        match selected_item.get() {
            Some(selected_item) => {
                view! {
                    <div class="relative flex-shrink-0 w-1/4 aspect-[2/3]">
                        <ItemCard item_specs=selected_item.item_specs.clone() />
                    </div>

                    <div class="flex-1 w-full">
                        <ItemTooltipContent item_specs=selected_item.item_specs.clone() />
                    </div>
                }
                .into_any()
            }
            None => {
                view! {
                    <div class="relative flex-shrink-0 w-1/4 aspect-[2/3]">
                        <div class="
                        relative group flex items-center justify-center w-full h-full
                        rounded-md border-2 border-zinc-700 bg-gradient-to-br from-zinc-800 to-zinc-900 opacity-70
                        "></div>
                    </div>

                    <div class="flex-1 text-gray-400">"No Item Selected"</div>
                }.into_any()
            }
        }
    };

    view! {
        <div class="w-full h-full flex items-center justify-center">
            <div class="flex flex-row gap-6 items-center w-full
            bg-neutral-800 rounded-lg  ring-1 ring-zinc-950  p-2">{item_details}</div>
        </div>
    }
}
