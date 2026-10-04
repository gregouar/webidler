use leptos::{prelude::*, task::spawn_local};

use shared::{
    computations,
    data::{
        item::InventorySortType,
        stash::{Stash, StashItem, StashType},
    },
    http::client::{
        ExchangeGemsStashRequest, GetStashItemsRequest, InventorySortRequest, StashAction,
        StoreStashItemRequest, TakeStashItemRequest, UpgradeStashRequest,
    },
    types::ItemPrice,
};

use crate::components::{
    backend_client::BackendClient,
    shared::{
        bag_grid::{BagAction, BagConfig, BagGrid, BagSortButton},
        inventory::equipped_item_comparison,
        resources::{GemsCounter, GoldIcon},
    },
    town::TownContext,
    ui::{
        buttons::{MenuButton, TabButton},
        card::{CardHeader, CardInset, MenuCard},
        input::ValidatedInput,
        menu_panel::MenuPanel,
        number::format_number,
        toast::*,
    },
};

#[component]
pub fn StashPanel(open: RwSignal<bool>) -> impl IntoView {
    let town: TownContext = expect_context();
    let backend: BackendClient = expect_context();
    let toaster: Toasts = expect_context();
    let character_id = town.character.read_untracked().character_id;

    let stash_type = RwSignal::new(StashType::Character);
    let stash = Signal::derive(move || match stash_type.get() {
        StashType::Character => town.character_stash,
        StashType::User => town.user_stash,
        StashType::Market => unreachable!("market stash is not a storage tab"),
    });
    let items = RwSignal::new(Vec::<StashItem>::new());
    let sort_type = RwSignal::new(InventorySortType::default());
    let selected = RwSignal::new(None);

    let busy = RwSignal::new(false);
    let loading = RwSignal::new(false);
    let request_version = RwSignal::new(0u64);

    Effect::new(move || {
        town.inventory.track();
        town.open_inventory.track();
        selected.set(None);
    });
    // Gem changes do not require reloading item contents.
    let stash_contents = Memo::new(move |_| {
        let stash = stash.get().get();
        (stash.stash_id, stash.max_items, stash.items_amount)
    });

    Effect::new(move || {
        let is_open = open.get();
        stash_type.track();
        let (stash_id, capacity, _) = stash_contents.get();
        let sorting = sort_type.get();
        let version = request_version.get_untracked() + 1;
        request_version.set(version);
        selected.set(None);
        loading.set(is_open && capacity > 0);
        if !is_open || capacity == 0 {
            return;
        }
        spawn_local(async move {
            let response = backend
                .get_stash_items(
                    &GetStashItemsRequest {
                        character_id,
                        sort_type: sorting,
                    },
                    &stash_id,
                )
                .await;
            // A previous tab or sort request must never replace the current view.
            if request_version.try_get_untracked() != Some(version) {
                return;
            }
            match response {
                Ok(response) => items.set(response.items),
                Err(e) => show_toast(
                    toaster,
                    format!("Failed to load items: {e}"),
                    ToastVariant::Error,
                ),
            }
            loading.set(false);
        });
    });

    let store_disabled = Signal::derive(move || {
        let stash = stash.get().get();
        !open.get() || busy.get() || loading.get() || stash.items_amount >= stash.max_items
    });
    let take_disabled = Signal::derive(move || {
        !open.get()
            || busy.get()
            || loading.get()
            || town.inventory.read().bag.len() >= town.inventory.read().max_bag_size as usize
    });
    let transfer = Callback::new(move |(action, index): (StashAction, usize)| {
        let disabled = match action {
            StashAction::Store => store_disabled.get_untracked(),
            StashAction::Take => take_disabled.get_untracked(),
        };
        if disabled {
            return;
        }
        let target = stash.get_untracked();
        let stash_id = target.read_untracked().stash_id;
        let item_index = match action {
            StashAction::Store => index,
            StashAction::Take => {
                let Some(item_id) =
                    items.with_untracked(|items| items.get(index).map(|item| item.stash_item_id))
                else {
                    return;
                };
                item_id
            }
        };
        selected.set(None);
        busy.set(true);
        spawn_local(async move {
            let result = match action {
                StashAction::Store => backend
                    .store_stash_item(
                        &StoreStashItemRequest {
                            character_id,
                            item_index,
                        },
                        &stash_id,
                    )
                    .await
                    .map(|response| (response.inventory, response.stash)),
                StashAction::Take => backend
                    .take_stash_item(
                        &TakeStashItemRequest {
                            character_id,
                            item_index: item_index as u32,
                        },
                        &stash_id,
                    )
                    .await
                    .map(|response| (response.inventory, response.stash)),
            };
            match result {
                Ok((inventory, updated_stash)) => {
                    town.inventory.set(inventory);
                    target.set(updated_stash);
                }
                Err(error) => show_toast(
                    toaster,
                    format!("Failed to transfer item: {error}"),
                    ToastVariant::Error,
                ),
            }
            busy.set(false);
        });
    });

    let sort_both = Callback::new(move |sorting| {
        if busy.get_untracked() || loading.get_untracked() {
            return;
        }
        selected.set(None);
        sort_type.set(sorting);
        busy.set(true);
        spawn_local(async move {
            match backend
                .inventory_sort(&InventorySortRequest {
                    character_id,
                    sort_type: sorting,
                })
                .await
            {
                Ok(response) => town.inventory.set(response.inventory),
                Err(error) => show_toast(
                    toaster,
                    format!("Failed to sort inventory: {error}"),
                    ToastVariant::Error,
                ),
            }
            busy.set(false);
        });
    });
    let upgrade = Memo::new(move |_| computations::stash_upgrade(&stash.get().get()));
    let upgrade_disabled =
        Signal::derive(move || busy.get() || upgrade.get().1 > town.character.read().resource_gold);
    let do_upgrade = move |_| {
        if upgrade_disabled.get_untracked() {
            return;
        }
        let target = stash.get_untracked();
        let stash_type = target.read_untracked().stash_type;
        selected.set(None);
        busy.set(true);
        spawn_local(async move {
            match backend
                .upgrade_stash(&UpgradeStashRequest {
                    character_id,
                    stash_type,
                })
                .await
            {
                Ok(response) => {
                    target.set(response.stash);
                    town.character.write().resource_gold = response.resource_gold;
                }
                Err(error) => show_toast(
                    toaster,
                    format!("Failed to upgrade stash: {error}"),
                    ToastVariant::Error,
                ),
            }
            busy.set(false);
        });
    };
    let stash_items: Signal<_> = Memo::new(move |_| {
        items.with(|items| items.iter().map(|item| item.item_specs.clone()).collect())
    })
    .into();
    let stash_capacity = Signal::derive(move || stash.get().read().max_items);
    let common_bag = BagConfig {
        max_item_level: Signal::derive(move || town.character.read().max_area_level),
        comparable_item: Some(equipped_item_comparison(town.inventory)),
        selected,
        ..Default::default()
    };
    let inventory_bag = StoredValue::new(BagConfig {
        items: Memo::new(move |_| town.inventory.read().bag.clone()).into(),
        capacity: Signal::derive(move || town.inventory.read().max_bag_size as usize),
        menu_id: "inventory",
        context_tooltip_right: true,
        actions: vec![BagAction {
            disabled: Some(Callback::new(move |_| store_disabled.get())),
            right_click: true,
            ..BagAction::new("Store", move |index| {
                transfer.run((StashAction::Store, index))
            })
        }],
        ..common_bag.clone()
    });
    let stash_bag = StoredValue::new(BagConfig {
        items: stash_items,
        capacity: stash_capacity,
        menu_id: "stash",
        actions: vec![BagAction {
            disabled: Some(Callback::new(move |_| take_disabled.get())),
            right_click: true,
            ..BagAction::new("Take", move |index| {
                transfer.run((StashAction::Take, index))
            })
        }],
        ..common_bag
    });

    view! {
        <MenuPanel open>
            <MenuCard class="h-full" gap=false>
                <CardHeader title="Stash" on_close=move || open.set(false)>
                    <div class="flex self-end justify-center h-full ml-2 xl:ml-4 gap-2 xl:gap-4 w-full max-w-md mx-auto overflow-clip">
                        <TabButton
                            is_active=Signal::derive(move || {
                                stash_type.get() == StashType::Character
                            })
                            on:click=move |_| stash_type.set(StashType::Character)
                        >
                            "Character Stash"
                        </TabButton>
                        <TabButton
                            is_active=Signal::derive(move || {
                                stash_type.get() == StashType::User
                            })
                            on:click=move |_| stash_type.set(StashType::User)
                        >
                            "User Stash"
                        </TabButton>
                    </div>

                    <div class="flex-1"></div>

                    <div class="flex items-center gap-2 mb-2">
                        <BagSortButton
                            on_sort=sort_both
                            disabled=Signal::derive(move || busy.get() || loading.get())
                        />
                    </div>

                    <div class="flex-1"></div>

                    <div class="flex justify-end mb-2">
                        <MenuButton on:click=do_upgrade disabled=upgrade_disabled>
                            <span class="flex items-center gap-1">
                                {move || {
                                    if stash_capacity.get() == 0 { "Buy" } else { "Upgrade" }
                                }} " ("{move || format_number(upgrade.get().1).to_string()}
                                <GoldIcon />")"
                            </span>
                        </MenuButton>
                    </div>
                </CardHeader>
                <div class="grid grid-cols-2 gap-2 xl:gap-4 min-h-0 flex-1">
                    <CardInset class="min-w-0 min-h-0" pad=false>
                        <div class="flex shrink-0 items-center justify-between gap-2 px-2 xl:px-3 py-2">
                            <h2 class="text-shadow-lg/100 shadow-gray-950 text-amber-300 text-sm xl:text-base font-display font-bold leading-none tracking-tight">
                                "Inventory"
                            </h2>
                            <span class="text-zinc-400 text-xs xl:text-base">
                                {move || {
                                    format!(
                                        "({} / {})",
                                        town.inventory.read().bag.len(),
                                        town.inventory.read().max_bag_size,
                                    )
                                }}
                            </span>
                        </div>
                        <div class="min-h-0 flex-1 overflow-y-auto pb-2">
                            <BagGrid config=inventory_bag.get_value() compact=true />
                        </div>
                    </CardInset>
                    <div class="flex min-w-0 min-h-0 flex-col gap-2">
                        <CardInset class="min-h-0 flex-1" pad=false>
                            <div class="flex shrink-0 items-center justify-between gap-2 px-2 xl:px-3 py-2">
                                <h2 class="text-shadow-lg/100 shadow-gray-950 text-amber-300 text-sm xl:text-base font-display font-bold leading-none tracking-tight">
                                    {move || match stash_type.get() {
                                        StashType::Character => "Character Stash",
                                        StashType::User => "User Stash",
                                        StashType::Market => "Market Stash",
                                    }}
                                </h2>
                                <span class="text-zinc-400 text-xs xl:text-base">
                                    {move || {
                                        let stash = stash.get().get();
                                        format!("({} / {})", stash.items_amount, stash.max_items)
                                    }}
                                </span>
                            </div>
                            <div class="min-h-0 flex-1 overflow-y-auto pb-2">
                                <Show when=move || stash_capacity.get() == 0>
                                    <p class="p-4 text-center text-zinc-400">
                                        "Buy this stash to store items."
                                    </p>
                                </Show>
                                <Show when=move || { stash_capacity.get() > 0 }>
                                    <BagGrid config=stash_bag.get_value() compact=true />
                                </Show>
                            </div>
                        </CardInset>
                        <div class="px-2 pb-2">
                            {move || match stash_type.get() {
                                StashType::Character => {
                                    view! { <Gems stash=town.character_stash busy /> }.into_any()
                                }
                                StashType::User => {
                                    view! { <Gems stash=town.user_stash busy /> }.into_any()
                                }
                                StashType::Market => ().into_any(),
                            }}
                        </div>
                    </div>
                </div>
            </MenuCard>
        </MenuPanel>
    }
}

#[component]
fn Gems(stash: RwSignal<Stash>, busy: RwSignal<bool>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let toaster = expect_context::<Toasts>();

    let value = Signal::derive(move || stash.read().resource_gems);
    let amount = RwSignal::new(Some(None::<ItemPrice>));

    let do_take = {
        let character_id = town_context.character.read_untracked().character_id;
        move |_| {
            let stash_id = stash.read_untracked().stash_id;
            if let Some(amount) = amount.get() {
                let amount = amount.unwrap_or(
                    ItemPrice::try_new(stash.read_untracked().resource_gems).unwrap_or_default(),
                );
                busy.set(true);
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
                        busy.set(false);
                    }
                });
            }
        }
    };

    let do_store = {
        let character_id = town_context.character.read_untracked().character_id;
        move |_| {
            let stash_id = stash.read_untracked().stash_id;
            if let Some(amount) = amount.get() {
                let amount = amount.unwrap_or(
                    ItemPrice::try_new(town_context.character.read_untracked().resource_gems)
                        .unwrap_or_default(),
                );
                busy.set(true);
                spawn_local({
                    async move {
                        match backend
                            .exchange_gems_stash(
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
                                format!("Failed to store gems: {e}"),
                                ToastVariant::Error,
                            ),
                        }
                        busy.set(false);
                    }
                });
            }
        }
    };

    let disable_take =
        Signal::derive(move || busy.get() || value.get() == 0.0 || stash.read().max_items == 0);
    let disable_store = Signal::derive(move || busy.get() || stash.read().max_items == 0);

    view! {
        <div class="flex min-w-0 items-center gap-2">
            <GemsCounter value w_full=true />
            <MenuButton on:click=do_store disabled=disable_store>
                "Store"
            </MenuButton>
            <MenuButton on:click=do_take disabled=disable_take>
                "Take"
            </MenuButton>
            <div class="w-20 shrink-0 xl:w-28">
                <ValidatedInput
                    id="gems_amount"
                    input_type="number"
                    placeholder="All"
                    bind=amount
                />
            </div>
        </div>
    }
}
