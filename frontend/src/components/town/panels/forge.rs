use chrono::Utc;
use leptos::{prelude::*, task::spawn_local};
use shared::{
    computations,
    constants::GAMBLE_ITEM_CATEGORIES,
    data::{
        forge,
        item::{ItemCategory, ItemRarity},
        item_affix::AffixType,
        player::EquippedSlot,
    },
    http::client::{
        ForgeAffixOperation, ForgeAffixRequest, ForgeUpgradeRequest, GambleItemRequest,
        InventoryDeleteRequest,
    },
};
use std::sync::Arc;

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    shared::{inventory::loot_filter_category_to_str, resources::GemsIcon},
    town::{
        TownContext,
        items_browser::{ItemDetails, ItemsBrowser, SelectedItem, SelectedMarketItem},
    },
    ui::{
        buttons::{MenuButton, MenuButtonRed, TabButton},
        card::{CardHeader, CardInset, CardTitle, MenuCard},
        confirm::ConfirmContext,
        menu_panel::MenuPanel,
        toast::*,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ForgeTab {
    Affix,
    Gamble,
    UniqueUpgrade,
}

#[component]
pub fn ForgePanel(open: RwSignal<bool>) -> impl IntoView {
    let selected_item = RwSignal::new(SelectedItem::None);
    let gamble_category = RwSignal::new(None);
    let active_tab = RwSignal::new(ForgeTab::Affix);

    let switch_tab = move |new_tab| {
        selected_item.set(SelectedItem::None);
        active_tab.set(new_tab);
    };

    view! {
        <MenuPanel open=open>
            <MenuCard class="h-full">
                <CardHeader title="Forge" on_close=move || open.set(false)>
                    <div class="flex self-end justify-center h-full ml-2 xl:ml-4 gap-2 xl:gap-4 -mb-2 overflow-hidden">
                        <TabButton
                            is_active=Signal::derive(move || {
                                active_tab.get() == ForgeTab::Affix
                            })
                            on:click=move |_| { switch_tab(ForgeTab::Affix) }
                            class:py-3
                        >
                            <div class="mx-1">"Craft"</div>
                        </TabButton>
                        <TabButton
                            is_active=Signal::derive(move || {
                                active_tab.get() == ForgeTab::UniqueUpgrade
                            })
                            on:click=move |_| { switch_tab(ForgeTab::UniqueUpgrade) }
                            class:py-3
                        >
                            <div class="mx-1">"Empower"</div>
                        </TabButton>
                        <TabButton
                            is_active=Signal::derive(move || {
                                active_tab.get() == ForgeTab::Gamble
                            })
                            on:click=move |_| { switch_tab(ForgeTab::Gamble) }
                            class:py-3
                        >
                            <div class="mx-1 mb-2">"Gamble"</div>
                        </TabButton>
                    </div>
                    <div class="flex-1" />
                </CardHeader>

                <div class="grid grid-cols-2 gap-2 min-h-0 flex-1">
                    <CardInset class="w-full" pad=false>

                        {move || match active_tab.get() {
                            ForgeTab::Affix | ForgeTab::UniqueUpgrade => {
                                view! { <InventoryBrowser selected_item /> }.into_any()
                            }
                            ForgeTab::Gamble => {
                                view! { <GambleBrowser gamble_category /> }.into_any()
                            }
                        }}
                    </CardInset>

                    <CardInset class="w-full">
                        {move || match active_tab.get() {
                            ForgeTab::Affix => {
                                view! { <ForgeAffixDetails selected_item /> }.into_any()
                            }
                            ForgeTab::UniqueUpgrade => {
                                view! { <UpgradeUniqueDetails selected_item /> }.into_any()
                            }
                            ForgeTab::Gamble => {
                                view! { <GambleDetails selected_item gamble_category /> }.into_any()
                            }
                        }}
                    </CardInset>
                </div>
            </MenuCard>
        </MenuPanel>
    }
}

#[component]
fn InventoryBrowser(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let town_context = expect_context::<TownContext>();

    let items_list = Signal::derive({
        move || {
            town_context.inventory.with(|inventory| {
                inventory
                    .equipped_items()
                    .map(|(slot, item)| SelectedMarketItem {
                        index: slot.into(),
                        owner_id: None,
                        owner_name: None,
                        // owner_id: town_context.character.read_untracked().character_id,
                        // owner_name: town_context.character.read_untracked().name.clone(),
                        recipient: Some((
                            town_context.character.read_untracked().character_id,
                            "".into(),
                        )),
                        item_specs: Arc::new(*item.clone()),
                        price: 0.0,
                        rejected: false,
                        created_at: Utc::now(),
                        deleted_at: None,
                        deleted_by: None,
                    })
                    .chain(inventory.bag.iter().enumerate().map(|(index, item)| {
                        SelectedMarketItem {
                            index: index + 9,
                            // owner_id: town_context.character.read_untracked().character_id,
                            // owner_name: town_context.character.read_untracked().name.clone(),
                            owner_id: None,
                            owner_name: None,
                            recipient: None,
                            item_specs: Arc::new(item.clone()),
                            price: 0.0,
                            rejected: false,
                            created_at: Utc::now(),
                            deleted_at: None,
                            deleted_by: None,
                        }
                    }))
                    .collect::<Vec<_>>()
            })
        }
    });

    view! { <ItemsBrowser selected_item items_list /> }
}

#[component]
pub fn ForgeAffixDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend: BackendClient = expect_context();
    let town_context: TownContext = expect_context();
    let auth_context: AuthContext = expect_context();
    let toaster: Toasts = expect_context();
    let confirm_context: ConfirmContext = expect_context();

    let user_gems = move || town_context.character.read().resource_gems;

    let item_level = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(item) => item.item_specs.modifiers.level,
            _ => 0,
        })
    };

    let do_affix_operation = {
        let character_id = town_context.character.read_untracked().character_id;
        move |operation| {
            if let SelectedItem::InMarket(item) = selected_item.get() {
                spawn_local({
                    async move {
                        match backend
                            .forge_affix(
                                &auth_context.token(),
                                &ForgeAffixRequest {
                                    character_id,
                                    item_index: item.index as u32,
                                    operation,
                                },
                            )
                            .await
                        {
                            Ok(response) => {
                                let updated_item_specs = if item.index < 9 {
                                    response
                                        .inventory
                                        .equipped
                                        .get(&item.index.try_into().unwrap())
                                        .cloned()
                                        .and_then(|equipped_item| match equipped_item {
                                            EquippedSlot::MainSlot(item_specs) => Some(*item_specs),
                                            _ => None,
                                        })
                                } else {
                                    response
                                        .inventory
                                        .bag
                                        .get(item.index.saturating_sub(9))
                                        .cloned()
                                };

                                if let Some(updated_item_specs) = updated_item_specs {
                                    selected_item.try_set(SelectedItem::InMarket(
                                        SelectedMarketItem {
                                            item_specs: Arc::new(updated_item_specs),
                                            ..item
                                        },
                                    ));
                                }

                                town_context.inventory.set(response.inventory);
                                town_context.character.write().resource_gems =
                                    response.resource_gems;
                            }
                            Err(e) => show_toast(
                                toaster,
                                format!("Failed to forge item: {e}"),
                                ToastVariant::Error,
                            ),
                        }
                    }
                });
            }
        }
    };

    let try_add_affix = {
        let confirm_context = confirm_context.clone();
        move |affix_type| {
            let do_add_affix =
                Arc::new(move || do_affix_operation(ForgeAffixOperation::Add(affix_type)));
            if town_context.character.read_untracked().max_area_level < item_level() {
                (confirm_context
                        .confirm)(
                        "Your Character Power Level is lower than this item's level. Forging an Affix may make it unusable for your character. Continue?"
                            .to_string(),
                        do_add_affix.clone(),
                    );
            } else {
                do_add_affix();
            }
        }
    };

    let try_remove_affix = {
        let confirm_context = confirm_context.clone();
        move || {
            let do_remove_affix = Arc::new(move || do_affix_operation(ForgeAffixOperation::Remove));
            (confirm_context.confirm)(
                "Removing an affix is random and cannot be undone. Continue?".to_string(),
                do_remove_affix.clone(),
            );
        }
    };

    let is_equipped = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => selected_item.recipient.is_some(),
            _ => false,
        })
    };

    let affix_price = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(item) => {
                if item.item_specs.base.rarity == ItemRarity::Unique {
                    return None;
                }
                forge::affix_price(item.item_specs.modifiers.count_nonunique_affixes())
            }
            _ => None,
        })
    };

    let prefix_price = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(item) => {
                if item.item_specs.base.rarity == ItemRarity::Unique {
                    return None;
                }

                let prefixes = item.item_specs.modifiers.count_affixes(AffixType::Prefix);
                let suffixes = item.item_specs.modifiers.count_affixes(AffixType::Suffix);

                if prefixes == suffixes {
                    forge::affix_price(prefixes + suffixes)
                        .map(|price| price * forge::PREFIX_PRICE_FACTOR)
                } else {
                    None
                }
            }
            _ => None,
        })
    };

    let suffix_price = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(item) => {
                if item.item_specs.base.rarity == ItemRarity::Unique {
                    return None;
                }

                let prefixes = item.item_specs.modifiers.count_affixes(AffixType::Prefix);
                let suffixes = item.item_specs.modifiers.count_affixes(AffixType::Suffix);

                if suffixes == prefixes {
                    forge::affix_price(prefixes + suffixes)
                        .map(|price| price * forge::SUFFIX_PRICE_FACTOR)
                } else {
                    None
                }
            }
            _ => None,
        })
    };

    let remove_price = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(item) => {
                forge::remove_price(item.item_specs.modifiers.count_nonunique_affixes())
            }
            _ => None,
        })
    };

    view! {
        <div class="w-full h-full flex flex-col justify-between relative">
            <CardTitle>"Forge Item"</CardTitle>

            <div class="flex flex-col">
                <span class="text-pink-400 font-bold text-sm xl:text-base">
                    {move || is_equipped().then_some("Equipped Item")}
                </span>
                <ItemDetails selected_item show_affixes=true />
            </div>

            <div class="flex flex-col gap-1 xl:gap-2">
                <MenuButton
                    on:click={
                        let try_add_affix = try_add_affix.clone();
                        move |_| try_add_affix(None)
                    }
                    disabled=Signal::derive({
                        move || affix_price().map(|price| price > user_gems()).unwrap_or(true)
                    })
                    class:mb-1
                    class:xl:mb-2
                >
                    <div class="w-full flex justify-center items-center gap-1 text-gray-400 h-[2em]">
                        "Add random" <span class="text-white font-bold">"Affix"</span>
                        {move || {
                            affix_price()
                                .map(|price| {
                                    view! {
                                        "for "
                                        <span class="text-fuchsia-300 font-bold">{price}</span>
                                        <GemsIcon />
                                    }
                                })
                        }}
                    </div>
                </MenuButton>
                <MenuButton
                    on:click={
                        let try_add_affix = try_add_affix.clone();
                        move |_| try_add_affix(Some(AffixType::Prefix))
                    }
                    disabled=Signal::derive({
                        move || prefix_price().map(|price| price > user_gems()).unwrap_or(true)
                    })
                >
                    <div class="w-full flex justify-center items-center gap-1 text-gray-400 h-[2em]">
                        "Add random" <span class="text-white font-bold">"Prefix"</span>
                        {move || {
                            prefix_price()
                                .map(|price| {
                                    view! {
                                        "for "
                                        <span class="text-fuchsia-300 font-bold">{price}</span>
                                        <GemsIcon />
                                    }
                                })
                        }}
                    </div>
                </MenuButton>
                <MenuButton
                    on:click=move |_| try_add_affix(Some(AffixType::Suffix))
                    disabled=Signal::derive({
                        move || suffix_price().map(|price| price > user_gems()).unwrap_or(true)
                    })
                >
                    <div class="w-full flex justify-center items-center gap-1 text-gray-400 h-[2em]">
                        "Add random" <span class="text-white font-bold">"Suffix"</span>
                        {move || {
                            suffix_price()
                                .map(|price| {
                                    view! {
                                        "for "
                                        <span class="text-fuchsia-300 font-bold">{price}</span>
                                        <GemsIcon />
                                    }
                                })
                        }}
                    </div>
                </MenuButton>

                <MenuButton
                    on:click=move |_| try_remove_affix()
                    disabled=Signal::derive({
                        move || remove_price().map(|price| price > user_gems()).unwrap_or(true)
                    })
                    class:mt-1
                    class:xl:mt-2
                >
                    <div class="w-full flex justify-center items-center gap-1 text-gray-400 h-[2em]">
                        "Remove random" <span class="text-white font-bold">"Affix"</span>
                        {move || {
                            remove_price()
                                .map(|price| {
                                    view! {
                                        "for "
                                        <span class="text-fuchsia-300 font-bold">{price}</span>
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

#[component]
pub fn UpgradeUniqueDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend: BackendClient = expect_context();
    let town_context: TownContext = expect_context();
    let auth_context: AuthContext = expect_context();
    let toaster: Toasts = expect_context();

    let user_gems = move || town_context.character.read().resource_gems;

    let wrong_item = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(item) => item.item_specs.base.upgrade_levels.is_empty(),
            _ => true,
        })
    };

    let do_upgrade_item = {
        let character_id = town_context.character.read_untracked().character_id;
        move || {
            if let SelectedItem::InMarket(item) = selected_item.get() {
                spawn_local({
                    async move {
                        match backend
                            .forge_upgrade(
                                &auth_context.token(),
                                &ForgeUpgradeRequest {
                                    character_id,
                                    item_index: item.index as u32,
                                },
                            )
                            .await
                        {
                            Ok(response) => {
                                let updated_item_specs = if item.index < 9 {
                                    response
                                        .inventory
                                        .equipped
                                        .get(&item.index.try_into().unwrap())
                                        .cloned()
                                        .and_then(|equipped_item| match equipped_item {
                                            EquippedSlot::MainSlot(item_specs) => Some(*item_specs),
                                            _ => None,
                                        })
                                } else {
                                    response
                                        .inventory
                                        .bag
                                        .get(item.index.saturating_sub(9))
                                        .cloned()
                                };

                                if let Some(updated_item_specs) = updated_item_specs {
                                    selected_item.try_set(SelectedItem::InMarket(
                                        SelectedMarketItem {
                                            item_specs: Arc::new(updated_item_specs),
                                            ..item
                                        },
                                    ));
                                }

                                town_context.inventory.set(response.inventory);
                                town_context.character.write().resource_gems =
                                    response.resource_gems;
                            }
                            Err(e) => show_toast(
                                toaster,
                                format!("Failed to forge item: {e}"),
                                ToastVariant::Error,
                            ),
                        }
                    }
                });
            }
        }
    };

    let is_equipped = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(selected_item) => selected_item.recipient.is_some(),
            _ => false,
        })
    };

    let upgrade_price = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(item) => computations::upgrade_item_price(&item.item_specs),
            _ => None,
        })
    };

    view! {
        <div class="w-full h-full flex flex-col justify-between relative">
            <CardTitle>"Empower Unique Item"</CardTitle>

            <div class="flex flex-col">
                <span class="text-pink-400 font-bold text-sm xl:text-base">
                    {move || is_equipped().then_some("Equipped Item")}
                </span>
                <ItemDetails selected_item show_affixes=true />
            </div>

            <div class="flex flex-col gap-1 xl:gap-2">
                <MenuButton
                    on:click=move |_| do_upgrade_item()
                    disabled=Signal::derive({
                        move || upgrade_price().map(|price| price > user_gems()).unwrap_or(true)
                    })
                    class:mb-1
                    class:xl:mb-2
                >
                    <div class="w-full flex justify-center items-center gap-1 text-gray-400 h-[2em]">
                        {move || {
                            if wrong_item() {
                                view! { "Item cannot be empowered" }.into_any()
                            } else if selected_item.read().is_empty() {
                                view! { "Select a Unique item to empower" }.into_any()
                            } else {
                                let upgrade_price = upgrade_price();
                                match upgrade_price {
                                    Some(upgrade_price) => {
                                        view! {
                                            "Empower for "
                                            <span class="text-fuchsia-300 font-bold">
                                                {upgrade_price}
                                            </span>
                                            <GemsIcon />
                                        }
                                            .into_any()
                                    }
                                    None => {
                                        view! { "Item has reached its maximum potential" }
                                            .into_any()
                                    }
                                }
                            }
                        }}
                    </div>
                </MenuButton>

            </div>
        </div>
    }
}

#[component]
fn GambleBrowser(gamble_category: RwSignal<Option<ItemCategory>>) -> impl IntoView {
    let set_gamble_category = move |item_category| gamble_category.set(item_category);
    view! {
        <div class="w-full flex flex-col gap-1 p-2">
            {GAMBLE_ITEM_CATEGORIES
                .map(|item_category| {
                    view! {
                        <MenuButton
                            class:w-full
                            on:click=move |_| set_gamble_category(item_category)
                        >
                            {loot_filter_category_to_str(item_category)}
                        </MenuButton>
                    }
                })}

        </div>
    }
}

#[component]
pub fn GambleDetails(
    selected_item: RwSignal<SelectedItem>,
    gamble_category: RwSignal<Option<ItemCategory>>,
) -> impl IntoView {
    let backend: BackendClient = expect_context();
    let town_context: TownContext = expect_context();
    let auth_context: AuthContext = expect_context();
    let toaster: Toasts = expect_context();

    let user_gems = move || town_context.character.read().resource_gems;

    let gamble_price =
        move || computations::gamble_price(town_context.character.read().max_area_level);

    let do_gamble = {
        let character_id = town_context.character.read_untracked().character_id;
        move || {
            spawn_local({
                async move {
                    match backend
                        .gamble_item(
                            &auth_context.token(),
                            &GambleItemRequest {
                                character_id,
                                item_category: gamble_category.get(),
                            },
                        )
                        .await
                    {
                        Ok(response) => {
                            let updated_item_specs = { response.inventory.bag.last().cloned() };
                            if let Some(updated_item_specs) = updated_item_specs {
                                selected_item.try_set(SelectedItem::InMarket(SelectedMarketItem {
                                    item_specs: Arc::new(updated_item_specs),
                                    index: response.inventory.bag.len().saturating_sub(1),
                                    owner_id: None,
                                    owner_name: None,
                                    recipient: None,
                                    price: 0.0,
                                    rejected: false,
                                    created_at: Utc::now(),
                                    deleted_at: None,
                                    deleted_by: None,
                                }));
                            }

                            town_context.inventory.set(response.inventory);
                            town_context.character.write().resource_gems = response.resource_gems;
                        }
                        Err(e) => show_toast(
                            toaster,
                            format!("Failed to gamble item: {e}"),
                            ToastVariant::Error,
                        ),
                    }
                }
            });
        }
    };

    let do_sell_and_gamble = {
        let character_id = town_context.character.read_untracked().character_id;
        move || {
            if let SelectedItem::InMarket(selected_item) = selected_item.get() {
                spawn_local({
                    async move {
                        match backend
                            .inventory_delete(
                                &auth_context.token(),
                                &InventoryDeleteRequest {
                                    character_id,
                                    item_indexes: [selected_item.index as u8].into(),
                                },
                            )
                            .await
                        {
                            Ok(_) => do_gamble(),
                            Err(e) => show_toast(
                                toaster,
                                format!("Failed to sell item: {e}"),
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
            <CardTitle>"Gamble"</CardTitle>

            <div class="flex flex-col w-full">
                <div class="w-full flex items-center justify-center text-sm xl:text-base text-gray-400 font-bold gap-2">
                    "Gamble <<"
                    <span class="text-white">
                        {move || { loot_filter_category_to_str(gamble_category.get()) }}
                    </span> ">> for" <span class="text-fuchsia-300 font-bold">{gamble_price}</span>
                    <GemsIcon />
                </div>
                <ItemDetails selected_item show_affixes=true />
            </div>

            <div class="flex gap-1 xl:gap-2 w-full text-white mb-2">
                <MenuButton
                    on:click=move |_| do_gamble()
                    disabled=Signal::derive({ move || gamble_price() > user_gems() })
                    class:w-full
                >
                    {move || {
                        if selected_item.read().is_empty() { "Gamble" } else { "Keep & Retry" }
                    }}
                </MenuButton>

                <MenuButtonRed
                    on:click=move |_| do_sell_and_gamble()
                    disabled=Signal::derive({
                        move || { gamble_price() > user_gems() || selected_item.read().is_empty() }
                    })
                    class:w-full
                >
                    "Discard & Retry"
                </MenuButtonRed>
            </div>
        </div>
    }
}
