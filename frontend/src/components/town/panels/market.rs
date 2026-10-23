use leptos::{prelude::*, task::spawn_local};
use shared_chat::types::ChatChannel;
use std::sync::Arc;
use strum::IntoEnumIterator;
use uuid::uuid;

use shared::{
    computations,
    data::{
        item::{ItemCategory, ItemRarity},
        market::{MarketFilters, MarketItem, MarketOrderBy, MarketStatFilter},
        modifier::Modifier,
        skill::{DamageType, RestoreType, SkillType},
        stash::{Stash, StashType},
        stat_effect::{
            ArmorStatType, LuckyRollType, StatSkillEffectType, StatSkillFilter, StatStatusFilter,
            StatType, StatusDamageType,
        },
    },
    http::client::{
        BrowseMarketItemsRequest, BuyMarketItemRequest, EditMarketItemRequest,
        ExchangeGemsStashRequest, RejectMarketItemRequest, SellMarketItemRequest, StashAction,
        UpgradeStashRequest,
    },
    types::{ItemPrice, PaginationLimit, Username},
};

use crate::components::{
    backend_client::BackendClient,
    chat::chat_context::ChatContext,
    shared::{
        inventory::loot_filter_category_to_str,
        resources::{GemsCounter, GemsIcon, GoldIcon},
        tooltips::effects_tooltip::{format_flat_stat, format_multiplier_stat_name},
    },
    town::{
        TownContext,
        items_browser::{
            InventoryItemGrid, ItemDetails, ItemsBrowser, MarketItemsGrid, SelectedItem,
            SelectedMarketItem,
        },
    },
    ui::{
        Separator,
        buttons::{MenuButton, MenuButtonRed, TabButton},
        card::{CardHeader, CardInset, CardInsetTitle, MenuCard},
        checkbox::Checkbox,
        dropdown::{DropdownMenu, SearchableDropdownMenu},
        input::{Input, ValidatedInput},
        menu_panel::MenuPanel,
        number::{format_datetime, format_number},
        toast::*,
        tooltip::{StaticTooltip, StaticTooltipPosition},
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum MarketTab {
    Filters,
    Buy,
    Sell,
    Listings,
    Logs,
}

#[component]
pub fn MarketPanel(open: RwSignal<bool>) -> impl IntoView {
    let town_context: TownContext = expect_context();

    let active_tab = RwSignal::new(MarketTab::Buy);
    let selected_item = RwSignal::new(SelectedItem::None);

    let switch_tab = move |new_tab| {
        selected_item.set(SelectedItem::None);
        active_tab.set(new_tab);
    };

    Effect::new(move || {
        if open.get() || town_context.open_inventory.get() {
            selected_item.set(SelectedItem::None);
            town_context.selected_item_index.set(None);
        }
    });

    let stash = town_context.market_stash;

    let filters = RwSignal::new(MarketFilters {
        max_req_level: Some(town_context.character.read_untracked().max_area_level),
        ..Default::default()
    });

    let disable_sell = Signal::derive(move || town_context.market_stash.read().max_items == 0);

    view! {
        <MenuPanel open=open>
            <MenuCard class="h-full" gap=false>
                <CardHeader title="Market" on_close=move || open.set(false)>
                    <div class="flex self-end justify-center h-full ml-2 xl:ml-4 gap-2 xl:gap-4 w-full max-w-md mx-auto overflow-clip">
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
                            disabled=disable_sell
                        >
                            "Sell"
                        </TabButton>
                        <TabButton
                            is_active=Signal::derive(move || {
                                active_tab.get() == MarketTab::Listings
                            })
                            on:click=move |_| { switch_tab(MarketTab::Listings) }
                            disabled=disable_sell
                        >
                            "Listings"
                        </TabButton>
                        <TabButton
                            is_active=Signal::derive(move || {
                                active_tab.get() == MarketTab::Logs
                            })
                            on:click=move |_| { switch_tab(MarketTab::Logs) }
                            disabled=disable_sell
                        >
                            "Logs"
                        </TabButton>
                    </div>

                    <div class="flex-1"></div>

                    <div class="flex items-center gap-2 mb-2">
                        <RevenueGems stash />
                    </div>

                    <div class="flex-1"></div>

                    <div class="flex shrink-0 justify-end mb-2 mr-2 xl:mr-4">
                        <MarketStashUpgradeButton />
                    </div>
                    <span class="text-shadow-md shadow-gray-950 text-zinc-400 text-xs xl:text-base font-medium">
                        {move || {
                            format!("({} / {})", stash.read().items_amount, stash.read().max_items)
                        }}
                    </span>
                </CardHeader>

                <div class="grid grid-cols-2 gap-2 xl:gap-4 min-h-0 flex-1">
                    <CardInset class="w-full" pad=false>
                        {move || {
                            match active_tab.get() {
                                MarketTab::Filters => view! { <MainFilters filters /> }.into_any(),
                                MarketTab::Buy => {
                                    view! { <MarketBrowser selected_item filters /> }.into_any()
                                }
                                MarketTab::Sell => {
                                    view! { <InventoryItemGrid selected_item /> }.into_any()
                                }
                                MarketTab::Listings => {
                                    view! { <MarketListings selected_item /> }.into_any()
                                }
                                MarketTab::Logs => view! { <MarketLogs selected_item /> }.into_any(),
                            }
                        }}
                    </CardInset>

                    <CardInset class="w-full">
                        {move || {
                            match active_tab.get() {
                                MarketTab::Filters => view! { <StatsFilters filters /> }.into_any(),
                                MarketTab::Buy => view! { <BuyDetails selected_item /> }.into_any(),
                                MarketTab::Sell => {
                                    view! { <SellDetails selected_item /> }.into_any()
                                }
                                MarketTab::Listings => {
                                    view! { <ListingDetails selected_item /> }.into_any()
                                }
                                MarketTab::Logs => {
                                    view! { <LogsDetails selected_item /> }.into_any()
                                }
                            }
                        }}
                    </CardInset>
                </div>
            </MenuCard>
        </MenuPanel>
    }
}

impl From<MarketItem> for SelectedMarketItem {
    fn from(value: MarketItem) -> Self {
        Self {
            index: value.item_id,
            item_specs: Arc::new(value.item_specs),
            price: value.price,
            owner_id: Some(value.owner_id),
            owner_name: Some(value.owner_name),
            recipient: value.recipient,
            rejected: value.rejected,
            created_at: value.created_at,
            deleted_at: value.deleted_at,
            deleted_by: value.deleted_by,
        }
    }
}

pub fn item_rarity_str(item_rarity: Option<ItemRarity>) -> &'static str {
    match item_rarity {
        None => "Any",
        Some(ItemRarity::Normal) => "Common",
        Some(ItemRarity::Magic) => "Magical",
        Some(ItemRarity::Rare) => "Rare",
        Some(ItemRarity::Masterwork) => "Soulforged",
        Some(ItemRarity::Unique) => "Unique",
    }
}

#[component]
pub fn RevenueGems(stash: RwSignal<Stash>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let toaster = expect_context::<Toasts>();

    let value = Signal::derive(move || stash.read().resource_gems);

    let do_take = {
        let character_id = town_context.character.read_untracked().character_id;
        move |_| {
            let stash_id = stash.read_untracked().stash_id;
            let amount =
                ItemPrice::try_new(stash.read_untracked().resource_gems).unwrap_or_default();
            spawn_local({
                async move {
                    match backend
                        .exchange_gems_stash(
                            &ExchangeGemsStashRequest {
                                character_id,
                                amount,
                                stash_action: StashAction::Take,
                            },
                            &stash_id,
                        )
                        .await
                    {
                        Ok(response) => {
                            town_context.character.write().resource_gems = response.resource_gems;
                            stash.set(response.stash);
                        }
                        Err(e) => show_toast(
                            toaster,
                            format!("Failed to take gems: {e}"),
                            ToastVariant::Error,
                        ),
                    }
                }
            });
        }
    };

    let disabled = Signal::derive(move || value.get() == 0.0);

    view! {
        <div class="flex gap-2 items-center">
            <span class="hidden xl:inline text-zinc-400 text-xs xl:text-base font-medium">
                "Revenue:"
            </span>
            <GemsCounter value w_full=true />
            <MenuButton on:click=do_take disabled>
                "Take"
            </MenuButton>
        </div>
    }
}

#[component]
fn MarketStashUpgradeButton() -> impl IntoView {
    let town = expect_context::<TownContext>();
    let backend = expect_context::<BackendClient>();
    let toaster = expect_context::<Toasts>();
    let stash = town.market_stash;
    let busy = RwSignal::new(false);
    let upgrade = Memo::new(move |_| computations::stash_upgrade(&stash.get()));
    let disabled =
        Signal::derive(move || busy.get() || upgrade.get().1 > town.character.read().resource_gold);
    let do_upgrade = move |_| {
        if disabled.get_untracked() {
            return;
        }
        let character_id = town.character.read_untracked().character_id;
        busy.set(true);
        spawn_local(async move {
            match backend
                .upgrade_stash(&UpgradeStashRequest {
                    character_id,
                    stash_type: StashType::Market,
                })
                .await
            {
                Ok(response) => {
                    stash.set(response.stash);
                    town.character.write().resource_gold = response.resource_gold;
                }
                Err(error) => show_toast(
                    toaster,
                    format!("Failed to upgrade market stash: {error}"),
                    ToastVariant::Error,
                ),
            }
            busy.set(false);
        });
    };
    view! {
        <MenuButton on:click=do_upgrade disabled>
            <span class="flex items-center gap-1 whitespace-nowrap">
                {move || if stash.read().max_items == 0 { "Buy Stash" } else { "Upgrade Stash" }}
                " (" {move || format_number(upgrade.get().1).to_string()}<GoldIcon />")"
            </span>
        </MenuButton>
    }
}

#[component]
fn MarketListings(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let town = expect_context::<TownContext>();
    let backend = expect_context::<BackendClient>();
    let toaster = expect_context::<Toasts>();
    let items = RwSignal::new(Vec::<SelectedMarketItem>::new());
    let refresh = RwSignal::new(0u64);
    let request_version = RwSignal::new(0u64);
    let stash_contents = Memo::new(move |_| {
        let stash = town.market_stash.read();
        (stash.stash_id, stash.max_items, stash.items_amount)
    });
    // Price edits clear the selection; removals also need to refresh the stash.
    Effect::new(move || {
        if !matches!(selected_item.get(), SelectedItem::InMarket(_)) {
            refresh.update(|value| *value += 1);
        }
    });
    Effect::new(move || {
        refresh.track();
        let (stash_id, capacity, _) = stash_contents.get();
        let version = request_version.get_untracked() + 1;
        request_version.set(version);
        if capacity == 0 {
            items.set(Vec::new());
            return;
        }
        spawn_local(async move {
            let response = backend.get_market_stash_items(&stash_id).await;
            if request_version.try_get_untracked() != Some(version) {
                return;
            }
            match response {
                Ok(response) => items.set(response.items.into_iter().map(Into::into).collect()),
                Err(error) => show_toast(
                    toaster,
                    format!("Failed to load listings: {error}"),
                    ToastVariant::Error,
                ),
            }
        });
    });
    view! {
        <MarketItemsGrid
            selected_item
            items_list=items
            capacity=Signal::derive(move || town.market_stash.read().max_items)
        />
    }
}

#[component]
fn MarketBrowser(
    selected_item: RwSignal<SelectedItem>,
    #[prop(into)] filters: Signal<MarketFilters>,
) -> impl IntoView {
    view! { <MarketList selected_item filters logs=false /> }
}

#[component]
fn MarketLogs(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    view! {
        <MarketList
            selected_item
            logs=true
            filters=Signal::derive(|| MarketFilters {
                order_by: MarketOrderBy::Time,
                ..Default::default()
            })
        />
    }
}

#[component]
fn MarketList(
    selected_item: RwSignal<SelectedItem>,
    filters: Signal<MarketFilters>,
    logs: bool,
) -> impl IntoView {
    let items_per_page = PaginationLimit::try_new(10).unwrap_or_default();

    let items_list = RwSignal::new(Vec::new());

    let extend_list = RwSignal::new(0u32);
    let reached_end_of_list = RwSignal::new(false);
    let has_more = RwSignal::new(true);

    let refresh_list = move || {
        items_list.write().drain(..);
        extend_list.set(0);
        has_more.set(true);
        reached_end_of_list.set(true);
    };

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

    Effect::new({
        let backend: BackendClient = expect_context();
        let town_context: TownContext = expect_context();
        move || {
            if reached_end_of_list.get() && has_more.get_untracked() {
                let skip = extend_list.get_untracked();
                (*extend_list.write()) += items_per_page.into_inner() as u32;

                let user_id = town_context.character.read_untracked().character_id;
                let realm = town_context.character.read_untracked().realm;
                let filters = filters.get_untracked();

                spawn_local(async move {
                    let response = backend
                        .browse_market_items(&BrowseMarketItemsRequest {
                            user_id,
                            realm,
                            skip,
                            limit: items_per_page,
                            filters,
                            own_listings: logs,
                            is_deleted: logs,
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
        }
    });

    view! {
        <ItemsBrowser
            selected_item
            items_list
            reached_end_of_list
            has_more
            show_filter_hint=!logs
        />
    }
}

#[component]
pub fn BuyDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let chat_context: ChatContext = expect_context();
    let toaster = expect_context::<Toasts>();

    let own_item = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => {
                selected_item.owner_id.unwrap_or_default() == town_context.character.read().user_id
            }
            _ => false,
        })
    };

    let disabled = Signal::derive({
        let town_context = expect_context::<TownContext>();
        move || {
            !own_item()
                && selected_item.with(|selected_item| match selected_item {
                    SelectedItem::InMarket(selected_item) => {
                        selected_item.price > town_context.character.read().resource_gems
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
            SelectedItem::InMarket(selected_item) => {
                selected_item.owner_name.clone().unwrap_or_default()
            }
            _ => "".into(),
        })
    };

    let seller_id = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => selected_item.owner_id.unwrap_or_default(),
            _ => Default::default(),
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
        move |_| {
            if let SelectedItem::InMarket(item) = selected_item.get() {
                spawn_local({
                    async move {
                        match backend
                            .buy_market_item(&BuyMarketItemRequest {
                                character_id,
                                item_index: item.index as u32,
                            })
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
        }
    };

    let do_reject = {
        move |_| {
            if let SelectedItem::InMarket(item) = selected_item.get() {
                spawn_local({
                    async move {
                        match backend
                            .reject_market_item(&RejectMarketItemRequest {
                                item_index: item.index as u32,
                            })
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
        }
    };

    view! {
        <div class="w-full h-full flex flex-col justify-between">
            <CardInsetTitle>"Buy from Market"</CardInsetTitle>

            <div class="flex flex-col">
                <span class="text-pink-400 p-2 font-bold">
                    {move || private_offer().then_some("Private Offer")}
                </span>
                <ItemDetails selected_item show_affixes=true />
                <div class="flex justify-between items-center text-xs xl:text-sm text-zinc-400 p-2">
                    <span>
                        "Listed by: "
                        <span
                            class="cursor-pointer"
                            on:click=move |_| {
                                chat_context.opened.set(true);
                                chat_context.minimized.set(false);
                                chat_context.users_map.write().insert(seller_id(), seller_name());
                                chat_context.write_channel.set(ChatChannel::Whisper(seller_id()));
                                selected_item
                                    .with(|selected_item| {
                                        if let SelectedItem::InMarket(selected_item) = selected_item {
                                            chat_context
                                                .linked_item
                                                .set(Some(selected_item.item_specs.clone()));
                                        }
                                    });
                            }
                        >
                            {move || seller_name()}
                        </span>
                    </span>
                    <span>{move || listed_at().map(format_datetime)}</span>
                </div>
            </div>

            <div class="w-full">
                <Separator />
                <div class="flex justify-between items-center p-4">
                    <div class="flex items-center gap-1 text-lg text-zinc-400">
                        {move || {
                            price()
                                .map(|price| {
                                    if price > 0.0 {
                                        view! {
                                            "Price: "
                                            <span class="text-fuchsia-300 font-bold">
                                                {format!("{:.0}", price)}
                                            </span>
                                            <GemsIcon />
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <span class="text-fuchsia-300 font-bold">"Free"</span>
                                        }
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
                        {move || {
                            if own_item() {
                                "Remove Item"
                            } else if price().unwrap_or(1.0) > 0.0 {
                                "Buy Item"
                            } else {
                                "Take Item"
                            }
                        }}
                    </MenuButton>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn SellDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let toaster = expect_context::<Toasts>();

    let price = RwSignal::new(None::<ItemPrice>);
    let recipient_name = RwSignal::new(Some(None::<Username>));

    let disabled = Signal::derive(move || {
        selected_item.read().is_empty() || price.read().is_none() || recipient_name.read().is_none()
    });

    let do_sell = {
        let character_id = town_context.character.read_untracked().character_id;
        move |_| {
            if let SelectedItem::InMarket(item) = selected_item.get() {
                let recipient_name = recipient_name.get().unwrap_or_default();
                let price = price.get().unwrap().into_inner();
                spawn_local({
                    async move {
                        match backend
                            .sell_market_item(&SellMarketItemRequest {
                                character_id,
                                recipient_name,
                                item_index: item.index,
                                price,
                            })
                            .await
                        {
                            Ok(response) => {
                                town_context.market_stash.set(response.stash);
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
        }
    };

    view! {
        <div class="w-full h-full flex flex-col justify-between relative">
            <CardInsetTitle>"Sell from Bag"</CardInsetTitle>

            <ValidatedInput
                id="private_offer"
                label="Private Offer:"
                input_type="text"
                placeholder="Enter User Name"
                bind=recipient_name
            />

            <ItemDetails selected_item show_affixes=true />

            <div class="w-full">
                <Separator />
                <div class="flex justify-between items-end p-1 xl:p-4">
                    <div class="flex items-end gap-1 text-lg text-zinc-400 ">
                        <ValidatedInput
                            id="price"
                            label="Price:"
                            input_type="number"
                            placeholder="Enter Price"
                            bind=price
                        />
                        <div class="flex items-center">
                            <GemsIcon />
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
        </div>
    }
}

#[component]
pub fn ListingDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
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
            SelectedItem::InMarket(selected_item) => {
                selected_item.owner_name.clone().unwrap_or_default()
            }
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
        move |_| {
            if let SelectedItem::InMarket(item) = selected_item.get() {
                let price = price.get_untracked().unwrap().into_inner();
                spawn_local({
                    async move {
                        match backend
                            .edit_market_item(&EditMarketItemRequest {
                                character_id,
                                item_index: item.index as u32,
                                price,
                            })
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
        }
    };

    let do_remove = {
        let character_id = town_context.character.read_untracked().character_id;
        move |_| {
            if let SelectedItem::InMarket(item) = selected_item.get() {
                spawn_local({
                    async move {
                        match backend
                            .buy_market_item(&BuyMarketItemRequest {
                                character_id,
                                item_index: item.index as u32,
                            })
                            .await
                        {
                            Ok(response) => {
                                town_context.market_stash.write().items_amount -= 1;
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
        }
    };

    view! {
        <div class="w-full h-full flex flex-col justify-between relative">
            <CardInsetTitle>"Remove from Market"</CardInsetTitle>

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
                <ItemDetails selected_item show_affixes=true />
                <div class="flex justify-between items-center text-sm text-zinc-400 p-2">
                    <span>"Listed by: " {move || seller_name()}</span>
                    <span>{move || listed_at().map(format_datetime)}</span>
                </div>
            </div>

            <div class="w-full">
                <Separator />
                <div class="flex justify-between items-end p-1 xl:p-4">
                    <div class="flex items-end gap-1 text-lg text-zinc-400 ">
                        <ValidatedInput
                            id="price"
                            label="Price:"
                            input_type="number"
                            placeholder="Enter Price"
                            bind=price
                        />
                        <div class="flex items-center">
                            <GemsIcon />
                        </div>
                        <MenuButton on:click=do_edit disabled=disabled>
                            <span class="inline xl:hidden">"Edit"</span>
                            <span class="hidden xl:inline font-variant:small-caps">
                                "Edit Price"
                            </span>
                        </MenuButton>
                    </div>

                    <MenuButton on:click=do_remove disabled=disabled>
                        <span class="inline xl:hidden">"Remove"</span>
                        <span class="hidden xl:inline font-variant:small-caps">"Remove Item"</span>
                    </MenuButton>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn LogsDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let price = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => Some(selected_item.price),
            _ => None,
        })
    };

    let removed = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => selected_item
                .deleted_by
                .as_ref()
                .map(|(deleted_by_id, _)| {
                    *deleted_by_id == town_context.character.read().character_id
                })
                .unwrap_or_default(),
            _ => false,
        })
    };

    let buyer_name = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => selected_item
                .deleted_by
                .as_ref()
                .map(|(_, deleted_by_name)| deleted_by_name.clone())
                .unwrap_or_default(),
            _ => "".into(),
        })
    };

    let bought_at = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => {
                Some(selected_item.deleted_at.unwrap_or_default())
            }
            _ => None,
        })
    };

    view! {
        <div class="w-full h-full flex flex-col justify-between relative">
            <CardInsetTitle>"Item Sold"</CardInsetTitle>

            <div class="flex flex-col">
                <ItemDetails selected_item show_affixes=true />
                <div class="flex justify-between items-center text-sm text-zinc-400 p-2">
                    {if removed() {
                        view! { <span>"Removed"</span> }.into_any()
                    } else {
                        view! {
                            <span>"Bought by: "{move || buyer_name()}</span>
                            <span>{move || bought_at().map(format_datetime)}</span>
                        }
                            .into_any()
                    }}
                </div>
            </div>

            <div class="w-full">
                <Separator />
                <div class="flex justify-between items-end p-4">
                    <div class="flex items-center gap-1 text-lg text-zinc-400">
                        {move || {
                            price()
                                .map(|price| {
                                    if price > 0.0 {
                                        view! {
                                            "Price: "
                                            <span class="text-fuchsia-300 font-bold">
                                                {format!("{:.0}", price)}
                                            </span>
                                            <GemsIcon />
                                        }
                                            .into_any()
                                    } else {
                                        view! {
                                            <span class="text-fuchsia-300 font-bold">"Free"</span>
                                        }
                                            .into_any()
                                    }
                                })
                        }}
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn MainFilters(filters: RwSignal<MarketFilters>) -> impl IntoView {
    let town_context: TownContext = expect_context();

    // Inputs
    macro_rules! rule_field {
        ($name:ident) => {
            let $name = RwSignal::new(None);

            Effect::new({
                move |_| {
                    $name.set(Some(filters.read().$name.clone()));
                }
            });

            Effect::new({
                move |_| {
                    if let Some(value) = $name.get()
                        && filters.read().$name != value
                    {
                        filters.write().$name = value;
                    }
                }
            });
        };
    }

    rule_field!(item_name);
    rule_field!(min_req_level);
    rule_field!(max_req_level);
    rule_field!(min_power_level);
    rule_field!(min_upgrade_level);
    rule_field!(min_max_power_shard_level);
    rule_field!(price);
    rule_field!(item_cooldown);
    rule_field!(item_damages);
    rule_field!(item_damage_physical);
    rule_field!(item_damage_fire);
    rule_field!(item_damage_poison);
    rule_field!(item_damage_storm);
    rule_field!(item_crit_chance);
    rule_field!(item_crit_damage);
    rule_field!(item_armor);
    rule_field!(item_block);

    // Dropdowns

    let item_rarity = RwSignal::new(None);
    Effect::new(move || item_rarity.set(filters.read().item_rarity));
    Effect::new(move || {
        let item_rarity = item_rarity.get();
        if filters.read_untracked().item_rarity != item_rarity {
            filters.write().item_rarity = item_rarity;
        }
    });
    let item_rarity_options = std::iter::once(None)
        .chain(ItemRarity::iter().map(Some))
        .map(|rarity| (rarity, item_rarity_str(rarity).into()))
        .collect();

    let item_category = RwSignal::new(None);
    Effect::new(move || item_category.set(filters.read().item_category));
    Effect::new(move || {
        let item_category = item_category.get();
        if filters.read_untracked().item_category != item_category {
            filters.write().item_category = item_category;
        }
    });
    let item_category_options = std::iter::once(None)
        .chain(
            ItemCategory::iter()
                .filter(|item_category| *item_category != ItemCategory::Cloak)
                .map(Some),
        )
        .map(|category| (category, loot_filter_category_to_str(category).into()))
        .collect();

    let order_by = RwSignal::new(Default::default());
    Effect::new(move || order_by.set(filters.read().order_by));
    Effect::new(move || {
        let order_by = order_by.get();
        if filters.read_untracked().order_by != order_by {
            filters.write().order_by = order_by;
        }
    });
    let order_by_options = MarketOrderBy::iter()
        .map(|category| {
            (
                category,
                match category {
                    MarketOrderBy::Price => "Lowest Price",
                    MarketOrderBy::Level => "Lowest Required Level",
                    MarketOrderBy::Damage => "Highest Damage",
                    MarketOrderBy::DamagePhysical => "Highest Physical Damage",
                    MarketOrderBy::DamageFire => "Highest Fire Damage",
                    MarketOrderBy::DamagePoison => "Highest Poison Damage",
                    MarketOrderBy::DamageStorm => "Highest Storm Damage",
                    MarketOrderBy::CritChance => "Highest Critical Hit Chance",
                    MarketOrderBy::CritDamage => "Highest Critical Hit Damage",
                    MarketOrderBy::Armor => "Highest Armor",
                    MarketOrderBy::Block => "Highest Block Chance",
                    MarketOrderBy::Time => "Most Recent",
                    MarketOrderBy::StatFilters => "Highest Stat Filters",
                }
                .into(),
            )
        })
        .collect();

    view! {
        <div class="w-full h-full flex flex-col gap-2 xl:gap-4 relative p-1 xl:p-4">
            <div class="flex gap-2 absolute top-3 right-3 z-10">
                <MenuButton on:click=move |_| {
                    let new_filters = MarketFilters {
                        max_req_level: Some(town_context.character.read_untracked().max_area_level),
                        stat_filters: filters.read_untracked().stat_filters.clone(),
                        ..Default::default()
                    };
                    *filters.write() = new_filters;
                }>"Reset"</MenuButton>
            </div>
            <CardInsetTitle>"Main Filters"</CardInsetTitle>

            <div class="grid grid-cols-1 xl:grid-cols-2 gap-4 p-4 border-b border-zinc-700">
                <div class="flex flex-col gap-4">
                    <ValidatedInput
                        id="item_name"
                        label="Item Name:"
                        input_type="text"
                        placeholder="Item Name substring"
                        bind=item_name
                    />

                    <div class="w-full flex gap-2">
                        <ValidatedInput
                            id="min_req_level"
                            label="Min Required Level:"
                            input_type="number"
                            placeholder="Min Required Level"
                            bind=min_req_level
                        />
                        <ValidatedInput
                            id="max_req_level"
                            label="Max Required Level:"
                            input_type="number"
                            placeholder="Max Required Level"
                            bind=max_req_level
                        />
                    </div>

                    <ValidatedInput
                        id="power_level"
                        label="Min Item Level:"
                        input_type="number"
                        placeholder="Minimum Item Level"
                        bind=min_power_level
                    />
                    <ValidatedInput
                        id="upgrade_level"
                        label="Min Empower Level:"
                        input_type="number"
                        placeholder="Minimum Empower Level"
                        bind=min_upgrade_level
                    />
                </div>

                <div class="flex flex-col gap-4">
                    <ValidatedInput
                        id="price"
                        label="Max Price:"
                        input_type="number"
                        placeholder="Maximum Price"
                        bind=price
                    />

                    <div class="flex items-center justify-between text-zinc-400 text-sm">
                        <span>"Item Category:"</span>
                        <SearchableDropdownMenu
                            options=item_category_options
                            chosen_option=item_category
                        />
                    </div>

                    <div class="flex items-center justify-between text-zinc-400 text-sm">
                        <span>"Item Rarity:"</span>
                        <DropdownMenu options=item_rarity_options chosen_option=item_rarity />
                    </div>

                    <div class="flex items-center justify-between text-zinc-400 text-sm">
                        <span>"Order by:"</span>
                        <DropdownMenu options=order_by_options chosen_option=order_by />
                    </div>
                </div>
            </div>

            <div class="grid grid-cols-1 xl:grid-cols-2 gap-4 p-4">
                <div class="flex flex-col gap-4">
                    <ValidatedInput
                        id="item_damages"
                        label="Min Damage:"
                        input_type="number"
                        placeholder="Minimum Damage per second"
                        bind=item_damages
                    />
                    <ValidatedInput
                        id="item_damage_physical"
                        label="Min Physical Damage:"
                        input_type="number"
                        placeholder="Minimum Physical Damage"
                        bind=item_damage_physical
                    />
                    <ValidatedInput
                        id="item_damage_fire"
                        label="Min Fire Damage:"
                        input_type="number"
                        placeholder="Minimum Fire Damage"
                        bind=item_damage_fire
                    />
                    <ValidatedInput
                        id="item_damage_poison"
                        label="Min Poison Damage:"
                        input_type="number"
                        placeholder="Minimum Poison Damage"
                        bind=item_damage_poison
                    />
                    <ValidatedInput
                        id="item_damage_storm"
                        label="Min Storm Damage:"
                        input_type="number"
                        placeholder="Minimum Storm Damage"
                        bind=item_damage_storm
                    />
                </div>
                <div class="flex flex-col gap-4">
                    <ValidatedInput
                        id="item_damages"
                        label="Max Cooldown:"
                        input_type="number"
                        placeholder="Maximum Cooldown"
                        bind=item_cooldown
                    />
                    <ValidatedInput
                        id="item_damages"
                        label="Min Critical Hit Chance:"
                        input_type="number"
                        placeholder="Minimum Critical Percent Chance"
                        bind=item_crit_chance
                    />
                    <ValidatedInput
                        id="item_damages"
                        label="Min Critical Hit Damage:"
                        input_type="number"
                        placeholder="Minimum extra Critical Percent Damage"
                        bind=item_crit_damage
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
                        placeholder="Minimum Block Percent Chance"
                        bind=item_block
                    />
                    <ValidatedInput
                        id="max_power_shard_level"
                        label="Min Power Shards Unlock Level:"
                        input_type="number"
                        placeholder="Minimum Power Shards Unlock Level"
                        bind=min_max_power_shard_level
                    />
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn StatsFilters(filters: RwSignal<MarketFilters>) -> impl IntoView {
    let stat_filters = filters.get_untracked().stat_filters.map(|stat_filter| {
        (
            RwSignal::new(
                stat_filter
                    .as_ref()
                    .map(|stat_filter| (stat_filter.stat.clone(), stat_filter.modifier)),
            ),
            RwSignal::new(
                stat_filter
                    .as_ref()
                    .and_then(|stat_filter| stat_filter.value),
            ),
            RwSignal::new(
                stat_filter
                    .as_ref()
                    .map(|stat_filter| stat_filter.exclude)
                    .unwrap_or_default(),
            ),
        )
    });

    Effect::new(move || {
        for (i, (stat_type, stat_value, stat_excluded)) in stat_filters.iter().enumerate() {
            filters.write().stat_filters[i] =
                stat_type.get().map(|(stat, modifier)| MarketStatFilter {
                    stat,
                    modifier,
                    value: stat_value.get(),
                    exclude: stat_excluded.get(),
                })
        }
    });

    view! {
        <div class="w-full min-h-0 flex-1 flex flex-col gap-2 xl:gap-4 relative">
            <CardInsetTitle>"Stat Filters"</CardInsetTitle>

            <div class="flex flex-col gap-2 xl:gap-4 p-2 xl:p-4">
                {stat_filters
                    .map(|(stat_type, stat_value, stat_excluded)| {
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
                                                        stat_excluded.set(false);
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
                                                        placeholder="Value"
                                                        bind=stat_value
                                                    />
                                                </div>
                                                <StaticTooltip
                                                    position=StaticTooltipPosition::Top
                                                    tooltip=|| {
                                                        "Require the stat to be absent or below the given value."
                                                    }
                                                >
                                                    <Checkbox
                                                        label="Exclude".to_string()
                                                        checked=stat_excluded
                                                        on_change=move |checked| { stat_excluded.set(checked) }
                                                    />
                                                </StaticTooltip>
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
pub fn StatDropdown(chosen_option: RwSignal<Option<(StatType, Modifier)>>) -> impl IntoView {
    let available_stats = vec![
        (StatType::Life, Modifier::Increased),
        (StatType::Life, Modifier::Flat),
        (StatType::LifeRegen, Modifier::Flat),
        (StatType::Mana, Modifier::Increased),
        (StatType::Mana, Modifier::Flat),
        (StatType::ManaRegen, Modifier::Flat),
        (StatType::Armor(Some(ArmorStatType::Fire)), Modifier::Flat),
        (StatType::Armor(Some(ArmorStatType::Poison)), Modifier::Flat),
        (StatType::Armor(Some(ArmorStatType::Storm)), Modifier::Flat),
        (
            StatType::Armor(Some(ArmorStatType::Elemental)),
            Modifier::Flat,
        ),
        (StatType::Armor(None), Modifier::Increased),
        (StatType::Evade(None), Modifier::Flat),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: None,
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::Damage {
                skill_filter: StatSkillFilter {
                    skill_type: Some(SkillType::Attack),
                    ..Default::default()
                },
                damage_type: None,
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::Damage {
                skill_filter: StatSkillFilter {
                    skill_type: Some(SkillType::Attack),
                    ..Default::default()
                },
                damage_type: None,
                min_max: None,
                is_hit: None,
            },
            Modifier::Increased,
        ),
        (
            StatType::Damage {
                skill_filter: StatSkillFilter {
                    skill_type: Some(SkillType::Spell),
                    ..Default::default()
                },
                damage_type: None,
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::Damage {
                skill_filter: StatSkillFilter {
                    skill_type: Some(SkillType::Spell),
                    ..Default::default()
                },
                damage_type: None,
                min_max: None,
                is_hit: None,
            },
            Modifier::Increased,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageType::Physical),
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageType::Physical),
                min_max: None,
                is_hit: None,
            },
            Modifier::Increased,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageType::Fire),
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageType::Fire),
                min_max: None,
                is_hit: None,
            },
            Modifier::Increased,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageType::Poison),
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageType::Poison),
                min_max: None,
                is_hit: None,
            },
            Modifier::Increased,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageType::Storm),
                min_max: None,
                is_hit: None,
            },
            Modifier::More,
        ),
        (
            StatType::Damage {
                skill_filter: Default::default(),
                damage_type: Some(DamageType::Storm),
                min_max: None,
                is_hit: None,
            },
            Modifier::Increased,
        ),
        (
            StatType::CritDamage(Default::default()),
            Modifier::Increased,
        ),
        (
            StatType::CritChance(Default::default()),
            Modifier::Increased,
        ),
        (
            StatType::CritChance(StatSkillFilter {
                skill_type: Some(SkillType::Spell),
                ..Default::default()
            }),
            Modifier::Increased,
        ),
        (
            StatType::StatusPower {
                status_filter: StatStatusFilter {
                    status_id: None,
                    damage_type: Some(StatusDamageType::Any),
                    debuff: None,
                },
                skill_filter: Default::default(),
                min_max: None,
            },
            Modifier::More,
        ),
        (
            StatType::StatusPower {
                status_filter: Default::default(),
                skill_filter: StatSkillFilter {
                    skill_type: Some(SkillType::Curse),
                    ..Default::default()
                },
                min_max: None,
            },
            Modifier::Increased,
        ),
        (
            StatType::StatusDuration {
                status_filter: StatStatusFilter {
                    debuff: Some(true),
                    ..Default::default()
                },
                skill_filter: Default::default(),
            },
            Modifier::Increased,
        ),
        (
            StatType::StatusDuration {
                status_filter: StatStatusFilter {
                    debuff: Some(false),
                    ..Default::default()
                },
                skill_filter: Default::default(),
            },
            Modifier::Increased,
        ),
        (
            StatType::SuccessChance {
                skill_filter: Default::default(),
                effect_type: Some(StatSkillEffectType::ApplyStatus {
                    status_id: None,
                    debuff: Some(true),
                }),
            },
            Modifier::Increased,
        ),
        (
            StatType::Restore {
                restore_type: Some(RestoreType::Life),
                skill_filter: Default::default(),
                modifier_description: Default::default(),
            },
            Modifier::Increased,
        ),
        (StatType::Speed(Default::default()), Modifier::Increased),
        (
            StatType::Speed(StatSkillFilter {
                skill_type: Some(SkillType::Attack),
                ..Default::default()
            }),
            Modifier::Increased,
        ),
        (
            StatType::Speed(StatSkillFilter {
                skill_type: Some(SkillType::Spell),
                ..Default::default()
            }),
            Modifier::Increased,
        ),
        (
            StatType::ManaCost {
                skill_filter: StatSkillFilter {
                    skill_type: Some(SkillType::Spell),
                    ..Default::default()
                },
            },
            Modifier::Increased,
        ),
        (StatType::MovementSpeed, Modifier::Increased),
        (StatType::GoldFind, Modifier::More),
        (StatType::ThreatGain, Modifier::Increased),
        (
            StatType::RestoreOnHit {
                restore_type: RestoreType::Life,
                skill_type: Some(SkillType::Attack),
            },
            Modifier::Flat,
        ),
        (
            StatType::RestoreOnHit {
                restore_type: RestoreType::Mana,
                skill_type: Some(SkillType::Attack),
            },
            Modifier::Flat,
        ),
        (StatType::SkillLevel(Default::default()), Modifier::Flat),
        (StatType::ItemRarity, Modifier::Increased),
        (StatType::ItemAreaChance, Modifier::Increased),
        (StatType::ItemLevel, Modifier::Flat),
        (StatType::GemsFind, Modifier::Increased),
        // Unique souls
        (
            StatType::Lucky {
                skill_filter: Default::default(),
                roll_type: LuckyRollType::Evade(None),
            },
            Modifier::Flat,
        ),
        // Unique passive souls
        (
            StatType::AllocatePassive(uuid!("0bd6d73c-0b10-4524-b280-ca71fdc054c9")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("0bf01c1a-5a2e-4d84-919a-7e4d1ca16fc8")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("17ccb40c-fe56-450b-9723-c12510d0c5c6")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("214090ea-2db1-429a-b965-382e8a2e78cb")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("2c872ce5-d750-4a2b-bdc9-ecf51501a7d1")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("3a4861a5-e8e3-455d-b458-fe23d932b190")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("4102c91f-9037-4b56-b695-e7c4448bd345")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("453ed12d-79ee-421f-aaca-fc21f58dc2d9")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("48348840-47ac-4112-a2a9-699d44fbc8c6")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("4993a125-fdbd-4ee2-a85e-d44f2c00e489")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("5124756b-5066-45d3-a219-3a581d7c8bb7")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("5223c739-dd9e-413d-804a-6b14ea39c1f7")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("56639962-c2b7-4135-868b-07486cbfc109")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("5f96379e-5249-4b0e-8a4d-1e8c0f951f82")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("665e7afa-d2aa-4075-bd54-278fe4673434")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("6afbaa18-6923-4d9f-af6e-d0edf482b786")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("6c91ed30-7f6b-4748-beab-6a64fc58be18")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("6f7434a6-2cfe-4325-8231-481f3792a47e")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("73c40387-2ca1-4169-9033-ec6e4c7d709f")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("765bb65d-4b0e-4904-9e5e-d100e174a2cd")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("7e10ea6e-22f5-43f9-8c49-9df5d664e87a")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("7fc83e8b-550b-47a9-952c-10b7ef15a0cd")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("84587682-6d87-4179-8c45-05ed5e640c38")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("91c8b2cf-977c-48ef-9bae-ffb5b20569a7")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("9d418005-cc92-4524-8b96-ad8a7b7ede57")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("a009d872-fa46-498a-af0c-3889e207631b")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("a2beb5ab-9c0c-485f-b5b1-f8aa631c95df")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("a796f363-7d58-4ebd-b78d-8f4cf92614bd")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("abd16719-d299-423a-a184-8cd233dfe87e")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("ad85ce7b-2f6b-4949-af76-6b6cad6ed862")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("c351443b-35e6-4860-aa43-86eec4c92e51")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("c6d1985a-af8b-4c62-a5f6-8afd826f5457")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("c85b224f-526d-4a9f-a2e4-63fd36cba963")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("cf14e084-a8ed-496a-b832-b3180ff93a51")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("cfff8b24-fb03-4a7e-a0b1-0c512a2ec7c4")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("d103b2b7-0801-4c7d-ad6d-fc735f713026")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("d1140822-66ab-4fbb-ba98-b1aced236696")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("d17092aa-df88-47bd-834f-1233404609da")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("d47c084d-d9f2-479a-b1a8-f538845af398")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("db23ff19-242a-4751-b628-e41b698a0788")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("eb5d66c8-17cc-4812-bcda-c425f0941aea")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("f275006b-3e0a-4177-a128-f501b7b91b4e")),
            Modifier::Flat,
        ),
        (
            StatType::AllocatePassive(uuid!("ff796aa2-b773-47aa-9745-622c333e424a")),
            Modifier::Flat,
        ),
    ];

    let options = available_stats
        .into_iter()
        .map(|(stat_type, modifier)| {
            (
                Some((stat_type.clone(), modifier)),
                format_stat_filter(&stat_type, modifier),
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

fn format_stat_filter(stat_type: &StatType, modifier: Modifier) -> String {
    match modifier {
        Modifier::Increased => format!("#% Increased {}", format_multiplier_stat_name(stat_type)),
        Modifier::More => format!("#% More {}", format_multiplier_stat_name(stat_type)),
        Modifier::Flat => format_flat_stat(stat_type, None),
    }
}
