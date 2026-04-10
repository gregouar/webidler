use chrono::Utc;
use leptos::{prelude::*, task::spawn_local};
use std::sync::Arc;

use shared::{
    computations,
    data::{
        market::MarketFilters,
        stash::{Stash, StashItem, StashType},
    },
    http::client::{
        BrowseStashItemsRequest, ExchangeGemsStashRequest, StashAction, StoreStashItemRequest,
        TakeStashItemRequest, UpgradeStashRequest,
    },
    types::{ItemPrice, PaginationLimit},
};

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    shared::{
        inventory::InventoryEquipFilter,
        resources::{GemsCounter, GoldIcon},
    },
    town::{
        TownContext,
        items_browser::{ItemDetails, ItemsBrowser, SelectedItem, SelectedMarketItem},
        panels::market::{MainFilters, StatsFilters},
    },
    ui::{
        Separator,
        buttons::{MenuButton, TabButton},
        card::{CardHeader, CardInset, CardInsetTitle, MenuCard},
        input::ValidatedInput,
        list_row::MenuListRow,
        menu_panel::MenuPanel,
        number::{format_datetime, format_number},
        toast::*,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum StashTab {
    Filters,
    Store,
    Take,
    BuyStash,
}

#[component]
pub fn StashPanel(open: RwSignal<bool>) -> impl IntoView {
    let town_context: TownContext = expect_context();
    let stash = town_context.user_stash;

    let selected_item = RwSignal::new(SelectedItem::None);
    let selected_stash = RwSignal::new(None);

    let filters = RwSignal::new(MarketFilters {
        // item_level: Some(town_context.character.read_untracked().max_area_level),
        ..Default::default()
    });

    let disable_stash = Signal::derive(move || stash.read().max_items == 0);

    let active_tab = RwSignal::new(if disable_stash.get_untracked() {
        StashTab::BuyStash
    } else {
        StashTab::Store
    });

    let switch_tab = move |new_tab| {
        selected_item.set(SelectedItem::None);
        active_tab.set(new_tab);
    };

    view! {
        <MenuPanel open=open>
            <MenuCard class="h-full" gap=false>
                <CardHeader title="Stash" on_close=move || open.set(false)>
                    <div class="flex self-end justify-center h-full ml-2 xl:ml-4 gap-2 xl:gap-4 w-full max-w-md mx-auto overflow-hidden">
                        <TabButton
                            is_active=Signal::derive(move || {
                                active_tab.get() == StashTab::Filters
                            })
                            on:click=move |_| { switch_tab(StashTab::Filters) }
                        >
                            "Filters"
                        </TabButton>
                        <TabButton
                            is_active=Signal::derive(move || {
                                active_tab.get() == StashTab::Store
                            })
                            on:click=move |_| { switch_tab(StashTab::Store) }
                            disabled=disable_stash
                        >
                            "Store"
                        </TabButton>
                        <TabButton
                            is_active=Signal::derive(move || { active_tab.get() == StashTab::Take })
                            on:click=move |_| { switch_tab(StashTab::Take) }
                            disabled=disable_stash
                        >
                            "Take"
                        </TabButton>
                        <TabButton
                            is_active=Signal::derive(move || {
                                active_tab.get() == StashTab::BuyStash
                            })
                            on:click=move |_| { switch_tab(StashTab::BuyStash) }
                        >
                            "Upgrade"
                        </TabButton>
                    </div>

                    <div class="flex-1"></div>
                    <div class="flex items-center gap-2 mb-2">
                        <Gems stash />
                    </div>
                    <div class="flex-1"></div>
                    <span class="text-shadow-md shadow-gray-950 text-gray-400 text-xs xl:text-base font-medium">
                        {move || {
                            format!("({} / {})", stash.read().items_amount, stash.read().max_items)
                        }}
                    </span>
                </CardHeader>

                <div class="grid grid-cols-2 gap-2 min-h-0 flex-1">
                    <CardInset class="w-full" pad=false>
                        {move || {
                            match active_tab.get() {
                                StashTab::Filters => view! { <MainFilters filters /> }.into_any(),
                                StashTab::Take => {
                                    view! { <StashBrowser stash selected_item filters /> }
                                        .into_any()
                                }
                                StashTab::Store => {
                                    view! { <InventoryBrowser selected_item /> }.into_any()
                                }
                                StashTab::BuyStash => {
                                    view! { <SelectBuyStash selected_stash /> }.into_any()
                                }
                            }
                        }}
                    </CardInset>

                    <CardInset class="w-full">
                        {move || {
                            match active_tab.get() {
                                StashTab::Filters => view! { <StatsFilters filters /> }.into_any(),
                                StashTab::Take => {
                                    view! { <TakeDetails stash selected_item /> }.into_any()
                                }
                                StashTab::Store => {
                                    view! { <StoreDetails stash selected_item /> }.into_any()
                                }
                                StashTab::BuyStash => {
                                    view! { <UpgradeStashDetails selected_stash /> }.into_any()
                                }
                            }
                        }}
                    </CardInset>
                </div>
            </MenuCard>
        </MenuPanel>
    }
}

impl From<StashItem> for SelectedMarketItem {
    fn from(value: StashItem) -> Self {
        Self {
            index: value.stash_item_id,
            item_specs: Arc::new(value.item_specs),
            price: 0.0,
            owner_id: value.character_id,
            owner_name: value.character_name,
            recipient: None,
            rejected: false,
            created_at: value.created_at,
            deleted_at: None,
            deleted_by: None,
        }
    }
}

#[component]
fn SelectBuyStash(selected_stash: RwSignal<Option<Stash>>) -> impl IntoView {
    let town_context: TownContext = expect_context();
    view! {
        <div class="gap-2 p-1 xl:p-2 flex flex-col">
            <StashTypeRow stash=town_context.user_stash selected_stash />
            <StashTypeRow stash=town_context.market_stash selected_stash />
        </div>
    }
}

fn stash_type_str(stash_type: StashType) -> &'static str {
    match stash_type {
        StashType::User => "User Stash",
        StashType::Market => "Market Stash",
    }
}

#[component]
fn StashTypeRow(stash: RwSignal<Stash>, selected_stash: RwSignal<Option<Stash>>) -> impl IntoView {
    let is_selected = Signal::derive(move || {
        selected_stash
            .read()
            .as_ref()
            .map(|selected_stash| {
                selected_stash.stash_id == stash.read().stash_id
                    && selected_stash.stash_type == stash.read().stash_type
            })
            .unwrap_or_default()
    });

    view! {
        <MenuListRow
            class="mb-2"
            selected=is_selected
            on_click=move || { selected_stash.set(Some(stash.get())) }
        >
            <div class="flex flex-col flex-1 gap-1 p-3">
                <div class="flex items-center justify-between">
                    <div class="text-lg font-semibold text-white capitalize">
                        {stash_type_str(stash.read_untracked().stash_type)}
                    </div>

                    <div class="text-sm text-gray-400">
                        {move || {
                            if stash.read().max_items > 0 {
                                format!("{}/{}", stash.read().items_amount, stash.read().max_items)
                            } else {
                                "Click to buy!".into()
                            }
                        }}
                    </div>
                </div>
            </div>
        </MenuListRow>
    }
}

#[component]
fn UpgradeStashDetails(selected_stash: RwSignal<Option<Stash>>) -> impl IntoView {
    let town_context = expect_context::<TownContext>();
    let backend = expect_context::<BackendClient>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let current_max_size = move || {
        selected_stash
            .read()
            .as_ref()
            .map(|selected_stash| selected_stash.max_items)
            .unwrap_or_default()
    };

    let upgrade = Signal::derive(move || {
        selected_stash.with(|selected_stash| {
            selected_stash
                .as_ref()
                .map(computations::stash_upgrade)
                .unwrap_or_default()
        })
    });

    let disabled =
        Signal::derive(move || upgrade.get().1 > town_context.character.read().resource_gold);

    let do_upgrade = {
        let character_id = town_context.character.read_untracked().character_id;
        move |_| {
            if let Some(stash) = selected_stash.get() {
                let stash_type = stash.stash_type;
                spawn_local({
                    async move {
                        match backend
                            .upgrade_stash(
                                &auth_context.token(),
                                &UpgradeStashRequest {
                                    character_id,
                                    stash_type,
                                },
                            )
                            .await
                        {
                            Ok(response) => {
                                selected_stash.set(Some(response.stash.clone()));
                                match response.stash.stash_type {
                                    StashType::User => town_context.user_stash.set(response.stash),
                                    StashType::Market => {
                                        town_context.market_stash.set(response.stash)
                                    }
                                };
                                town_context.character.write().resource_gold =
                                    response.resource_gold;
                            }
                            Err(e) => show_toast(
                                toaster,
                                format!("Failed to upgrade stash: {e}"),
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
            <CardInsetTitle>"Upgrade Stashes"</CardInsetTitle>

            <div class="flex flex-col gap-2">
                <div class="text-lg font-semibold text-white capitalize">
                    {move || {
                        if let Some(selected_stash) = selected_stash.get() {
                            stash_type_str(selected_stash.stash_type)
                        } else {
                            ""
                        }
                    }}
                </div>
                <div class="relative isolate overflow-hidden rounded-[8px] border border-[#3b3428]
                bg-[linear-gradient(180deg,rgba(226,193,122,0.05),rgba(0,0,0,0.02)_28%,rgba(0,0,0,0.14)_100%),linear-gradient(135deg,rgba(40,39,45,0.98),rgba(18,18,22,1))]
                shadow-[0_4px_12px_rgba(0,0,0,0.24),inset_0_1px_0_rgba(255,255,255,0.04),inset_0_-1px_0_rgba(0,0,0,0.35)] p-3">
                    <div class="pointer-events-none absolute inset-[1px] rounded-[7px] border border-white/5"></div>
                    <div class="pointer-events-none absolute inset-x-3 top-0 h-px bg-gradient-to-r from-transparent via-[#edd39a]/35 to-transparent"></div>
                    <div class="relative z-10">
                        <div class="text-xs text-gray-400 mb-1">"Current"</div>
                        <div class="text-blue-400 font-medium">
                            {move || {
                                selected_stash
                                    .read()
                                    .as_ref()
                                    .map(|selected_stash| {
                                        if selected_stash.max_items > 0 {
                                            format!("Storage Space: {}", selected_stash.max_items)
                                        } else {
                                            "".into()
                                        }
                                    })
                                    .unwrap_or_default()
                            }}
                        </div>
                    </div>
                </div>

                <div class="relative isolate overflow-hidden rounded-[8px] border border-[#3b3428]
                bg-[linear-gradient(180deg,rgba(226,193,122,0.05),rgba(0,0,0,0.02)_28%,rgba(0,0,0,0.14)_100%),linear-gradient(135deg,rgba(40,39,45,0.98),rgba(18,18,22,1))]
                shadow-[0_4px_12px_rgba(0,0,0,0.24),inset_0_1px_0_rgba(255,255,255,0.04),inset_0_-1px_0_rgba(0,0,0,0.35)] p-3">
                    <div class="pointer-events-none absolute inset-[1px] rounded-[7px] border border-white/5"></div>
                    <div class="pointer-events-none absolute inset-x-3 top-0 h-px bg-gradient-to-r from-transparent via-[#edd39a]/35 to-transparent"></div>
                    <div class="relative z-10">
                        <div class="text-xs text-gray-400 mb-1">"Next"</div>
                        <div class="text-blue-400 font-medium">
                            {move || { format!("Storage Space: {}", upgrade.get().0) }}
                        </div>
                    </div>
                </div>
            </div>

            <div class="w-full">
                <Separator />
                <div class="flex justify-between items-center p-4">
                    <div class="flex items-center gap-1 text-lg text-gray-400">
                        {move || {
                            view! {
                                "Price: "
                                <span class="text-amber-300 font-bold">
                                    {format_number(upgrade.get().1).to_string()}
                                </span>
                                <GoldIcon />
                            }
                        }}
                    </div>
                    <MenuButton on:click=do_upgrade disabled=disabled>
                        {move || {
                            if current_max_size() == 0 { "Buy Stash" } else { "Upgrade Stash" }
                        }}
                    </MenuButton>
                </div>
            </div>
        </div>
    }
}

#[component]
fn StashBrowser(
    stash: RwSignal<Stash>,
    selected_item: RwSignal<SelectedItem>,
    #[prop(into)] filters: Signal<MarketFilters>,
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
        let backend = expect_context::<BackendClient>();
        let town_context = expect_context::<TownContext>();
        let auth_context = expect_context::<AuthContext>();
        move || {
            if reached_end_of_list.get() && has_more.get_untracked() {
                let skip = extend_list.get_untracked();
                (*extend_list.write()) += items_per_page.into_inner() as u32;

                let character_id = town_context.character.read_untracked().character_id;
                let stash_id = stash.read_untracked().stash_id;
                let filters = filters.get_untracked();

                spawn_local(async move {
                    let response = backend
                        .browse_stash_items(
                            &auth_context.token(),
                            &BrowseStashItemsRequest {
                                character_id,
                                skip,
                                limit: items_per_page,
                                filters,
                            },
                            &stash_id,
                        )
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

    view! { <ItemsBrowser selected_item items_list reached_end_of_list has_more /> }
}

#[component]
fn InventoryBrowser(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let town_context: TownContext = expect_context();

    let select_from_inventory = move |_| {
        town_context.selected_item_index.set(None);
        town_context.equip_filter.set(InventoryEquipFilter::Bag);
        town_context.open_inventory.set(true);
    };

    // TODO: Direct send stash request?
    Effect::new(move || {
        if let Some(item_index) = town_context.selected_item_index.get() {
            let item_specs = town_context
                .inventory
                .read()
                .bag
                .get(item_index as usize)
                .cloned();

            if let Some(item_specs) = item_specs {
                selected_item.set(SelectedItem::InMarket(SelectedMarketItem {
                    index: item_index as usize,
                    item_specs: Arc::new(item_specs),
                    price: 0.0,
                    owner_id: None,
                    owner_name: None,
                    recipient: None,
                    rejected: false,
                    created_at: Utc::now(),
                    deleted_at: None,
                    deleted_by: None,
                }));
            }
        }
    });

    let items_list = Signal::derive(move || {
        town_context
            .inventory
            .read()
            .bag
            .iter()
            .enumerate()
            .map(|(index, item)| SelectedMarketItem {
                index,
                owner_id: None,
                owner_name: None,
                // owner_id: Some(town_context.character.read_untracked().character_id),
                // owner_name: Some(town_context.character.read_untracked().name.clone()),
                recipient: None,
                item_specs: Arc::new(item.clone()),
                price: 0.0,
                rejected: false,
                created_at: Utc::now(),
                deleted_at: None,
                deleted_by: None,
            })
            .collect::<Vec<_>>()
    });

    view! {
        <div class="w-full px-2 pt-2">
            <MenuButton class="w-full" on:click=select_from_inventory>
                "Pick from Inventory"
            </MenuButton>
        </div>
        <ItemsBrowser selected_item items_list />
    }
}

#[component]
pub fn TakeDetails(stash: RwSignal<Stash>, selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let disabled = Signal::derive({
        // let town_context = expect_context::<TownContext>();s
        move || {
            selected_item.with(|selected_item| !matches!(selected_item, SelectedItem::InMarket(_)))
        }
    });

    let owner_name = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => {
                selected_item.owner_name.clone().unwrap_or_default()
            }
            _ => "".into(),
        })
    };

    let stored_at = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => Some(selected_item.created_at),
            _ => None,
        })
    };

    let do_take = {
        let character_id = town_context.character.read_untracked().character_id;
        let stash_id = stash.read_untracked().stash_id;
        move |_| {
            if let SelectedItem::InMarket(item) = selected_item.get() {
                spawn_local({
                    async move {
                        match backend
                            .take_stash_item(
                                &auth_context.token(),
                                &TakeStashItemRequest {
                                    character_id,
                                    item_index: item.index as u32,
                                },
                                &stash_id,
                            )
                            .await
                        {
                            Ok(response) => {
                                town_context.inventory.set(response.inventory);
                                town_context.user_stash.set(response.stash);
                                selected_item.set(SelectedItem::Removed(item.index));
                            }
                            Err(e) => show_toast(
                                toaster,
                                format!("Failed to take item: {e}"),
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
            <CardInsetTitle>"Take from Stash"</CardInsetTitle>

            <div class="flex flex-col">
                <ItemDetails selected_item show_affixes=true />
                <div class="flex justify-between items-center text-sm text-gray-400 p-2">
                    <span>"Stored by: "{move || owner_name()}</span>
                    <span>{move || stored_at().map(format_datetime)}</span>
                </div>
            </div>

            <div class="w-full">
                <Separator />
                <div class="flex justify-end items-center p-4">
                    <MenuButton on:click=do_take disabled=disabled>
                        "Take Item"
                    </MenuButton>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn StoreDetails(
    stash: RwSignal<Stash>,
    selected_item: RwSignal<SelectedItem>,
) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let disabled = Signal::derive(move || selected_item.read().is_empty());

    let do_store = {
        let character_id = town_context.character.read_untracked().character_id;
        let stash_id = stash.read_untracked().stash_id;
        move |_| {
            if let SelectedItem::InMarket(item) = selected_item.get() {
                spawn_local({
                    async move {
                        match backend
                            .store_stash_item(
                                &auth_context.token(),
                                &StoreStashItemRequest {
                                    character_id,
                                    item_index: item.index,
                                },
                                &stash_id,
                            )
                            .await
                        {
                            Ok(response) => {
                                town_context.inventory.set(response.inventory);
                                town_context.user_stash.set(response.stash);
                                selected_item.set(SelectedItem::Removed(item.index));
                            }
                            Err(e) => show_toast(
                                toaster,
                                format!("Failed to store item: {e}"),
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
            <CardInsetTitle>"Store from Inventory"</CardInsetTitle>

            <ItemDetails selected_item show_affixes=true />

            <div class="w-full">
                <Separator />
                <div class="flex justify-end items-end p-4">
                    <MenuButton on:click=do_store disabled=disabled>
                        "Store Item"
                    </MenuButton>
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Gems(stash: RwSignal<Stash>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let value = Signal::derive(move || stash.read().resource_gems);
    let amount = RwSignal::new(Some(None::<ItemPrice>));

    let do_take = {
        let character_id = town_context.character.read_untracked().character_id;
        let stash_id = stash.read_untracked().stash_id;
        move |_| {
            if let Some(amount) = amount.get() {
                let amount = amount.unwrap_or(
                    ItemPrice::try_new(stash.read_untracked().resource_gems).unwrap_or_default(),
                );

                spawn_local({
                    async move {
                        match backend
                            .exchange_gems_stash(
                                &auth_context.token(),
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
                                town_context.character.write().resource_gems =
                                    response.resource_gems;
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
        }
    };

    let do_store = {
        let character_id = town_context.character.read_untracked().character_id;
        let stash_id = stash.read_untracked().stash_id;
        move |_| {
            if let Some(amount) = amount.get() {
                let amount = amount.unwrap_or(
                    ItemPrice::try_new(town_context.character.read_untracked().resource_gems)
                        .unwrap_or_default(),
                );
                spawn_local({
                    async move {
                        match backend
                            .exchange_gems_stash(
                                &auth_context.token(),
                                &ExchangeGemsStashRequest {
                                    character_id,
                                    amount,
                                    stash_action: StashAction::Store,
                                },
                                &stash_id,
                            )
                            .await
                        {
                            Ok(response) => {
                                town_context.character.write().resource_gems =
                                    response.resource_gems;
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
        }
    };

    let disable_take = Signal::derive(move || value.get() == 0.0 || stash.read().max_items == 0);
    let disable_store = Signal::derive(move || stash.read().max_items == 0);

    view! {
        <div class="flex gap-2 items-center">
            <GemsCounter value w_full=true />
            <MenuButton on:click=do_store disabled=disable_store>
                "Store"
            </MenuButton>
            <MenuButton on:click=do_take disabled=disable_take>
                "Take"
            </MenuButton>
            <ValidatedInput id="gems_amount" input_type="number" placeholder="All" bind=amount />
        </div>
    }
}
