use chrono::Utc;
use leptos::{prelude::*, task::spawn_local};
use std::sync::Arc;

use shared::{
    data::market::{MarketFilters, StashItem},
    http::client::{BrowseStashItemsRequest, StoreStashItemRequest, TakeStashItemRequest},
    types::PaginationLimit,
};

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    town::{
        items_browser::{ItemDetails, ItemsBrowser, SelectedItem, SelectedMarketItem},
        panels::market::{MainFilters, StatsFilters},
        TownContext,
    },
    ui::{
        buttons::{CloseButton, MenuButton, TabButton},
        menu_panel::{MenuPanel, PanelTitle},
        number::format_datetime,
        toast::*,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum StashTab {
    Filters,
    Store,
    Take,
}

#[component]
pub fn StashPanel(open: RwSignal<bool>) -> impl IntoView {
    let active_tab = RwSignal::new(StashTab::Store);
    let selected_item = RwSignal::new(SelectedItem::None);

    let switch_tab = move |new_tab| {
        selected_item.set(SelectedItem::None);
        active_tab.set(new_tab);
    };

    let filters = RwSignal::new(MarketFilters {
        // item_level: Some(town_context.character.read_untracked().max_area_level),
        ..Default::default()
    });

    view! {
        <MenuPanel open=open>
            <div class="w-full">
                <div class="bg-zinc-800 rounded-md p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col">
                    <div class="px-4 relative z-10 flex items-center justify-between">
                        <PanelTitle>"User Stash"</PanelTitle>

                        <div class="flex-1 flex justify-center ml-2 xl:ml-4 gap-2 xl:gap-4 w-full max-w-md mx-auto">
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
                            >
                                "Store"
                            </TabButton>
                            <TabButton
                                is_active=Signal::derive(move || {
                                    active_tab.get() == StashTab::Take
                                })
                                on:click=move |_| { switch_tab(StashTab::Take) }
                            >
                                "Take"
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
                                    StashTab::Filters => {
                                        view! { <MainFilters filters /> }.into_any()
                                    }
                                    StashTab::Take => {
                                        view! { <StashBrowser selected_item filters /> }.into_any()
                                    }
                                    StashTab::Store => {
                                        view! { <InventoryBrowser selected_item /> }.into_any()
                                    }
                                }
                            }}
                        </div>

                        <div class="w-full aspect-[4/3] bg-neutral-900 overflow-y-auto shadow-[inset_0_0_32px_rgba(0,0,0,0.6)]">
                            {move || {
                                match active_tab.get() {
                                    StashTab::Filters => {
                                        view! { <StatsFilters filters /> }.into_any()
                                    }
                                    StashTab::Take => {
                                        view! { <TakeDetails selected_item /> }.into_any()
                                    }
                                    StashTab::Store => {
                                        view! { <StoreDetails selected_item /> }.into_any()
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

impl From<StashItem> for SelectedMarketItem {
    fn from(value: StashItem) -> Self {
        Self {
            index: value.item_id,
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
fn StashBrowser(
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
        move || {
            if reached_end_of_list.get() && has_more.get_untracked() {
                let skip = extend_list.get_untracked();
                (*extend_list.write()) += items_per_page.into_inner() as u32;

                let character_id = town_context.character.read_untracked().character_id;
                let filters = filters.get_untracked();

                spawn_local(async move {
                    let response = backend
                        .browse_stash_items(&BrowseStashItemsRequest {
                            character_id,
                            skip,
                            limit: items_per_page,
                            filters,
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
                    deleted_at: None,
                    deleted_by: None,
                })
                .collect::<Vec<_>>()
        }
    });

    view! { <ItemsBrowser selected_item items_list /> }
}

#[component]
pub fn TakeDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let disabled = Signal::derive({
        let town_context = expect_context::<TownContext>();
        move || {
            selected_item.with(|selected_item| match selected_item {
                SelectedItem::InMarket(selected_item) => {
                    selected_item.item_specs.required_level
                        > town_context.character.read().max_area_level
                }
                _ => true,
            })
        }
    });

    let owner_name = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => selected_item.owner_name.to_string(),
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
                            )
                            .await
                        {
                            Ok(response) => {
                                town_context.inventory.set(response.inventory);
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
        <div class="w-full h-full flex flex-col justify-between p-4 relative">
            <span class="text-xl font-semibold text-amber-200 text-shadow-md text-center mb-2">
                "Take from Stash"
            </span>

            <div class="flex flex-col">
                <ItemDetails selected_item show_affixes=true />
                <div class="flex justify-between items-center text-sm text-gray-400 p-2">
                    <span>"Stored by: "{move || owner_name()}</span>
                    <span>{move || stored_at().map(format_datetime)}</span>
                </div>
            </div>

            <div class="flex justify-end items-center p-4 border-t border-zinc-700">
                <MenuButton on:click=do_take disabled=disabled>
                    "Take Item"
                </MenuButton>
            </div>
        </div>
    }
}

#[component]
pub fn StoreDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let disabled = Signal::derive(move || selected_item.read().is_empty());

    let do_sell = {
        let character_id = town_context.character.read_untracked().character_id;
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
                            )
                            .await
                        {
                            Ok(response) => {
                                town_context.inventory.set(response.inventory);
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
        <div class="w-full h-full flex flex-col justify-between p-4 relative">
            <span class="text-xl font-semibold text-amber-200 text-shadow-md text-center">
                "Store from Inventory"
            </span>

            <ItemDetails selected_item show_affixes=true />

            <div class="flex justify-end items-end p-4 border-t border-zinc-700">
                <MenuButton on:click=do_sell disabled=disabled>
                    "Store Item"
                </MenuButton>
            </div>
        </div>
    }
}
