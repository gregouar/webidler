use chrono::{DateTime, Utc};
use leptos::{html::*, prelude::*, task::spawn_local};
use leptos_use::{use_infinite_scroll_with_options, UseInfiniteScrollOptions};
use std::sync::Arc;
use strum::IntoEnumIterator;

use shared::{
    data::{
        item::{ItemCategory, ItemRarity, ItemSpecs},
        market::{MarketFilters, MarketItem},
        user::UserCharacterId,
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
            item_card::ItemCard, panels::inventory::loot_filter_category_to_str,
            tooltips::item_tooltip::ItemTooltipContent,
        },
        town::TownContext,
        ui::{
            buttons::{CloseButton, MenuButton, MenuButtonRed, TabButton},
            dropdown::DropdownMenu,
            input::ValidatedInput,
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
    let active_tab = RwSignal::new(MarketTab::Buy); // TODO: Start on filters?
    let selected_item = RwSignal::new(None::<SelectedItem>);

    let switch_tab = move |new_tab| {
        selected_item.set(None);
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
            <div class="w-full p-4">
                <div class="bg-zinc-800 rounded-md p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col">
                    <div class="px-4 relative z-10 flex items-center justify-between">
                        <PanelTitle>"Market"</PanelTitle>

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

#[derive(Clone)]
pub struct SelectedItem {
    pub index: usize,
    pub item_specs: Arc<ItemSpecs>,
    pub price: f64,
    pub owner_id: UserCharacterId,
    pub owner_name: String,
    pub recipient: Option<(UserCharacterId, String)>,
    pub rejected: bool,
    pub created_at: DateTime<Utc>,
}

impl From<MarketItem> for SelectedItem {
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

    view! {
        <div class="w-full h-full flex flex-col p-4 relative">
            <span class="text-xl font-semibold text-amber-200 text-shadow-md text-center mb-2">
                "Main Filters"
            </span>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 p-4 border-b border-zinc-700">
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
                        <DropdownMenu options=item_category_options chosen_option=item_category />
                    </div>

                    <div class="flex items-center justify-between text-gray-300 text-sm">
                        <span>"Item Rarity:"</span>
                        <DropdownMenu options=item_rarity_options chosen_option=item_rarity />
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-1 lg:grid-cols-2 gap-4 p-4">
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
    // TODO

    let _ = filters;

    view! {
        <div class="w-full h-full flex flex-col justify-between p-4 relative">
            <span class="text-xl font-semibold text-amber-200 text-shadow-md text-center mb-2">
                "Stats Filters"
            </span>
            <div class="w-full h-full flex items-center justify-center">
                <div class="flex flex-col items-center text-center gap-1">
                    <span class="text-gray-400">"Coming soon..."</span>
                </div>
            </div>
        </div>
    }
}

#[component]
fn MarketBrowser(
    selected_item: RwSignal<Option<SelectedItem>>,
    filters: RwSignal<MarketFilters>,
    own_listings: bool,
) -> impl IntoView {
    let items_per_page = PaginationLimit::try_new(20).unwrap_or_default();

    let items_list = RwSignal::new(Vec::new());

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
fn InventoryBrowser(selected_item: RwSignal<Option<SelectedItem>>) -> impl IntoView {
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
                key=|item| (item.index,item.created_at)
                let:(item)
            >
                <ItemRow
                    item_specs=item.item_specs.clone()
                    on:click=move |_| selected_item.set(Some(item.clone()))
                    price=item.price
                    highlight=move || selected_item.read().as_ref().map(|selected_item| selected_item.index==item.index).unwrap_or_default()
                    special_offer=item.recipient.is_some()
                    rejected=item.rejected
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
    #[prop(default = false)] special_offer: bool,
    #[prop(default = false)] rejected: bool,
) -> impl IntoView {
    view! {
        <div class=move || {
            format!(
                "relative flex w-full items-center justify-between p-3 gap-2 cursor-pointer mb-2 shadow-sm transition-colors duration-150 rounded-lg
                bg-neutral-800 hover:bg-neutral-700 {} {} {}",
                if highlight() { "ring-2 ring-amber-400" } else { "ring-1 ring-zinc-950" },
                if special_offer { "border-2 border-pink-400" } else { "" },
                if rejected { "border-2 border-red-500" } else { "" },
            )
        }>
            <div class="relative h-32 aspect-[2/3] flex-shrink-0">
                <ItemCard item_specs=item_specs.clone() class:pointer-events-none />
            </div>

            <div class="flex flex-col w-full">
                <ItemTooltipContent item_specs />
            </div>

            {(price > 0.0)
                .then(|| {
                    view! {
                        <div class="absolute flex bottom-2 right-2 gap-1 items-center">
                            <span class="text-gray-400">"Price:"</span>
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
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let disabled = Signal::derive({
        let town_context = expect_context::<TownContext>();
        move || match selected_item.read().as_ref() {
            Some(selected_item) => {
                selected_item.price > town_context.character.read().resource_gems
                    || selected_item.item_specs.modifiers.level
                        > town_context.character.read().max_area_level
            }
            None => true,
        }
    });

    let price = move || {
        selected_item
            .read()
            .as_ref()
            .map(|selected_item| selected_item.price)
    };

    let private_offer = move || {
        selected_item
            .read()
            .as_ref()
            .map(|selected_item| selected_item.recipient.is_some())
            .unwrap_or_default()
    };

    let seller_name = move || {
        selected_item
            .read()
            .as_ref()
            .map(|selected_item| selected_item.owner_name.to_string())
    };

    let listed_at = move || {
        selected_item
            .read()
            .as_ref()
            .map(|selected_item| selected_item.created_at)
    };

    let do_buy = {
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
                                format!("Failed to buy item: {e}"),
                                ToastVariant::Error,
                            ),
                        }
                    }
                });
            }
        }
    };

    let do_reject = {
        let character_id = town_context.character.read_untracked().character_id;
        move |_| {
            if let Some(item) = selected_item.get() {
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
                                selected_item.set(None);
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
pub fn SellDetails(selected_item: RwSignal<Option<SelectedItem>>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let price = RwSignal::new(None::<ItemPrice>);
    let recipient_name = RwSignal::new(Some(None::<Username>));

    let disabled = Signal::derive(move || {
        selected_item.read().is_none() || price.read().is_none() || recipient_name.read().is_none()
    });

    let do_sell = {
        let character_id = town_context.character.read_untracked().character_id;
        move |_| {
            if let Some(item) = selected_item.get() {
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
                                selected_item.set(None);
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
pub fn ListingDetails(selected_item: RwSignal<Option<SelectedItem>>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let disabled = Signal::derive(move || selected_item.read().is_none());

    let price = RwSignal::new(ItemPrice::try_new(0.0).ok());

    Effect::new(move || {
        price.set(
            ItemPrice::try_new(match selected_item.read().as_ref() {
                Some(selected_item) => selected_item.price,
                None => 0.0,
            })
            .ok(),
        );
    });

    let recipient_name = move || {
        selected_item
            .read()
            .as_ref()
            .map(|selected_item| {
                selected_item
                    .recipient
                    .as_ref()
                    .map(|(_, recipient_name)| recipient_name.clone())
            })
            .unwrap_or_default()
    };

    let rejected = move || {
        selected_item
            .read()
            .as_ref()
            .map(|selected_item| selected_item.rejected)
            .unwrap_or_default()
    };

    let seller_name = move || {
        selected_item
            .read()
            .as_ref()
            .map(|selected_item| selected_item.owner_name.to_string())
    };

    let listed_at = move || {
        selected_item
            .read()
            .as_ref()
            .map(|selected_item| selected_item.created_at)
    };

    let do_edit = {
        let character_id = town_context.character.read_untracked().character_id;
        move |_| {
            if let Some(item) = selected_item.get() {
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
                                selected_item.set(None);
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
        }
    };

    let do_remove = {
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
                                format!("Failed to remove listing: {e}"),
                                ToastVariant::Error,
                            ),
                        }
                    }
                });
            }
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
pub fn ItemDetails(selected_item: RwSignal<Option<SelectedItem>>) -> impl IntoView {
    let item_details = move || {
        match selected_item.get() {
            Some(selected_item) => {
                view! {
                    <div class="relative flex-shrink-0 w-1/4 aspect-[2/3]">
                        <ItemCard
                            item_specs=selected_item.item_specs.clone()
                            class:pointer-events-none
                        />
                    </div>

                    <div class="flex-1 w-full">
                        <ItemTooltipContent
                            item_specs=selected_item.item_specs.clone()
                            class:select-text
                        />
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
