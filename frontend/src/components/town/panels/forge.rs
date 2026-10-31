use chrono::Utc;
use leptos::{prelude::*, task::spawn_local};
use shared::{
    computations,
    constants::GAMBLE_ITEM_CATEGORIES,
    data::{
        forge,
        item::{ItemCategory, ItemRarity},
        item_affix::AffixType,
    },
    http::client::{
        ForgeAffixOperation, ForgeAffixRequest, ForgeImbueRequest, ForgeSacrificeRequest,
        ForgeUpgradeRequest, GambleItemRequest, InventoryDeleteRequest, InventorySortRequest,
    },
};
use std::sync::Arc;

use crate::components::{
    backend_client::BackendClient,
    shared::{
        bag_grid::BagSortButton, inventory::loot_filter_category_to_str, resources::GemsIcon,
    },
    town::{
        TownContext,
        items_browser::{
            InventoryItemEligibility, InventoryItemGrid, ItemDetails, SelectedItem,
            SelectedMarketItem, inventory_grid_item,
        },
    },
    ui::{
        Separator,
        buttons::{MenuButton, MenuButtonRed, TabButton},
        card::{CardHeader, CardInset, CardInsetTitle, MenuCard},
        checkbox::Checkbox,
        confirm::ConfirmContext,
        dropdown::SearchableDropdownMenu,
        menu_panel::MenuPanel,
        toast::*,
        tooltip::HelpTooltip,
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum ForgeTab {
    Affix,
    Sacrifice,
    Imbue,
    Gamble,
    UniqueUpgrade,
}

#[component]
pub fn ForgePanel(open: RwSignal<bool>) -> impl IntoView {
    let backend: BackendClient = expect_context();
    let town_context: TownContext = expect_context();
    let toaster: Toasts = expect_context();
    let selected_item = RwSignal::new(SelectedItem::None);
    let selected_soul = RwSignal::new(SelectedItem::None);
    let gamble_category = RwSignal::new(None);
    let active_tab = RwSignal::new(ForgeTab::Affix);
    let sorting = RwSignal::new(false);

    let switch_tab = move |new_tab| {
        selected_item.set(SelectedItem::None);
        selected_soul.set(SelectedItem::None);
        town_context.selected_item_index.set(None);
        active_tab.set(new_tab);
    };

    Effect::new(move || {
        if open.get() || town_context.open_inventory.get() {
            selected_item.set(SelectedItem::None);
            selected_soul.set(SelectedItem::None);
            town_context.selected_item_index.set(None);
        }
    });

    let sort_inventory = Callback::new(move |sort_type| {
        if sorting.get_untracked() {
            return;
        }

        selected_item.set(SelectedItem::None);
        selected_soul.set(SelectedItem::None);
        town_context.selected_item_index.set(None);
        sorting.set(true);

        let character_id = town_context.character.read_untracked().character_id;
        spawn_local(async move {
            match backend
                .inventory_sort(&InventorySortRequest {
                    character_id,
                    sort_type,
                })
                .await
            {
                Ok(response) => town_context.inventory.set(response.inventory),
                Err(error) => show_toast(
                    toaster,
                    format!("Failed to sort inventory: {error}"),
                    ToastVariant::Error,
                ),
            }
            sorting.set(false);
        });
    });

    view! {
        <MenuPanel open=open>
            <MenuCard class="h-full">
                <CardHeader title="Forge" on_close=move || open.set(false)>
                    <div class="flex self-end justify-center h-full ml-2 xl:ml-4 gap-2 xl:gap-4 -mb-2 overflow-clip">
                        <TabButton
                            is_active=Signal::derive(move || {
                                active_tab.get() == ForgeTab::Affix
                            })
                            on:click=move |_| { switch_tab(ForgeTab::Affix) }
                            class:py-3
                        >
                            <div class="mx-1">"Forge"</div>
                        </TabButton>
                        <TabButton
                            is_active=Signal::derive(move || {
                                active_tab.get() == ForgeTab::Sacrifice
                            })
                            on:click=move |_| { switch_tab(ForgeTab::Sacrifice) }
                            class:py-3
                        >
                            <div class="mx-1">"Sacrifice"</div>
                        </TabButton>
                        <TabButton
                            is_active=Signal::derive(move || {
                                active_tab.get() == ForgeTab::Imbue
                            })
                            on:click=move |_| { switch_tab(ForgeTab::Imbue) }
                            class:py-3
                        >
                            <div class="mx-1">"Imbue"</div>
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
                            <div class="mx-1">"Gamble"</div>
                        </TabButton>
                    </div>
                    <div class="flex-1 mb-2" />
                    <div class="mb-2">
                        <BagSortButton
                            on_sort=sort_inventory
                            disabled=Signal::derive(move || sorting.get())
                        />
                    </div>
                </CardHeader>

                <div class="grid grid-cols-2 gap-2 xl:gap-4 min-h-0 flex-1">
                    <CardInset class="w-full" pad=false>

                        {move || match active_tab.get() {
                            ForgeTab::Affix | ForgeTab::UniqueUpgrade | ForgeTab::Sacrifice => {
                                let eligibility = match active_tab.get() {
                                    ForgeTab::Affix => InventoryItemEligibility::NonUnique,
                                    ForgeTab::UniqueUpgrade => InventoryItemEligibility::Unique,
                                    ForgeTab::Sacrifice => InventoryItemEligibility::Sacrifice,
                                    _ => InventoryItemEligibility::Any,
                                };
                                view! { <InventoryItemGrid selected_item eligibility /> }.into_any()
                            }
                            ForgeTab::Imbue => {
                                let on_item_select = Callback::new(move |item: SelectedMarketItem| {
                                    if item.item_specs.base.categories.contains(&ItemCategory::Soul)
                                    {
                                        selected_soul.set(SelectedItem::InMarket(item));
                                    } else {
                                        selected_item.set(SelectedItem::InMarket(item));
                                    }
                                });
                                let on_item_cancel = Callback::new(move |item: SelectedMarketItem| {
                                    if item.item_specs.base.categories.contains(&ItemCategory::Soul)
                                    {
                                        selected_soul.set(SelectedItem::None);
                                    } else {
                                        selected_item.set(SelectedItem::None);
                                    }
                                });
                                view! {
                                    <InventoryItemGrid
                                        selected_item
                                        eligibility=InventoryItemEligibility::Imbue
                                        on_item_select
                                        on_item_cancel
                                    />
                                }
                                    .into_any()
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
                            ForgeTab::Sacrifice => {
                                view! { <SacrificeDetails selected_item /> }.into_any()
                            }
                            ForgeTab::Imbue => {
                                view! { <ImbueDetails selected_item selected_soul /> }.into_any()
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
pub fn ForgeAffixDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend: BackendClient = expect_context();
    let town_context: TownContext = expect_context();
    let toaster: Toasts = expect_context();
    let confirm_context: ConfirmContext = expect_context();
    let auto_confirm_destructive_actions = RwSignal::new(false);

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
                            .forge_affix(&ForgeAffixRequest {
                                character_id,
                                item_index: item.index as u32,
                                operation,
                            })
                            .await
                        {
                            Ok(response) => {
                                let updated_item_specs =
                                    response.inventory.bag.get(item.index).cloned();

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

    let perform_replace_affix = {
        let character_id = town_context.character.read_untracked().character_id;
        move |affix_type| {
            if let SelectedItem::InMarket(item) = selected_item.get() {
                spawn_local(async move {
                    let remove_response = match backend
                        .forge_affix(&ForgeAffixRequest {
                            character_id,
                            item_index: item.index as u32,
                            operation: ForgeAffixOperation::Remove,
                        })
                        .await
                    {
                        Ok(response) => response,
                        Err(error) => {
                            show_toast(
                                toaster,
                                format!("Failed to replace affix: {error}"),
                                ToastVariant::Error,
                            );
                            return;
                        }
                    };

                    match backend
                        .forge_affix(&ForgeAffixRequest {
                            character_id,
                            item_index: item.index as u32,
                            operation: ForgeAffixOperation::Add(affix_type),
                        })
                        .await
                    {
                        Ok(response) => {
                            let updated_item_specs =
                                response.inventory.bag.get(item.index).cloned();

                            if let Some(updated_item_specs) = updated_item_specs {
                                selected_item.try_set(SelectedItem::InMarket(SelectedMarketItem {
                                    item_specs: Arc::new(updated_item_specs),
                                    ..item
                                }));
                            }

                            town_context.inventory.set(response.inventory);
                            town_context.character.write().resource_gems = response.resource_gems;
                        }
                        Err(error) => {
                            let updated_item_specs =
                                remove_response.inventory.bag.get(item.index).cloned();

                            if let Some(updated_item_specs) = updated_item_specs {
                                selected_item.try_set(SelectedItem::InMarket(SelectedMarketItem {
                                    item_specs: Arc::new(updated_item_specs),
                                    ..item
                                }));
                            }

                            town_context.inventory.set(remove_response.inventory);
                            town_context.character.write().resource_gems =
                                remove_response.resource_gems;
                            show_toast(
                                toaster,
                                format!(
                                    "Affix was removed, but adding its replacement failed: {error}"
                                ),
                                ToastVariant::Error,
                            );
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
            if auto_confirm_destructive_actions.get_untracked() {
                do_remove_affix();
            } else {
                (confirm_context.confirm)(
                    "Removing an affix is random and cannot be undone. Continue?".to_string(),
                    do_remove_affix,
                );
            }
        }
    };

    let try_replace_affix = {
        let confirm_context = confirm_context.clone();
        move |affix_type| {
            let do_replace_affix = Arc::new(move || perform_replace_affix(affix_type));
            if auto_confirm_destructive_actions.get_untracked() {
                do_replace_affix();
            } else {
                let message = if town_context.character.read_untracked().max_area_level
                    < item_level()
                {
                    "Replacing an affix is random and cannot be undone. The new affix may make this item unusable for your character. Continue?"
                } else {
                    "Replacing an affix is random and cannot be undone. Continue?"
                };
                (confirm_context.confirm)(message.to_string(), do_replace_affix);
            }
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
                forge::affix_operation_price(
                    ForgeAffixOperation::Add(None),
                    item.item_specs.modifiers.count_nonunique_affixes(),
                    &item.item_specs.base,
                )
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
                    forge::affix_operation_price(
                        ForgeAffixOperation::Add(Some(AffixType::Prefix)),
                        prefixes + suffixes,
                        &item.item_specs.base,
                    )
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
                    forge::affix_operation_price(
                        ForgeAffixOperation::Add(Some(AffixType::Suffix)),
                        prefixes + suffixes,
                        &item.item_specs.base,
                    )
                } else {
                    None
                }
            }
            _ => None,
        })
    };

    let remove_price = move || {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(item) => forge::affix_operation_price(
                ForgeAffixOperation::Remove,
                item.item_specs.modifiers.count_nonunique_affixes(),
                &item.item_specs.base,
            ),
            _ => None,
        })
    };

    let replace_price = move |affix_type| {
        selected_item.with(|selected_item| match selected_item {
            SelectedItem::InMarket(item) => {
                if item.item_specs.base.rarity == ItemRarity::Unique {
                    return None;
                }

                let prefixes = item.item_specs.modifiers.count_affixes(AffixType::Prefix);
                let suffixes = item.item_specs.modifiers.count_affixes(AffixType::Suffix);
                let available = match affix_type {
                    None => prefixes + suffixes > 0,
                    Some(AffixType::Prefix) => prefixes > suffixes,
                    Some(AffixType::Suffix) => suffixes > prefixes,
                    _ => false,
                };

                if !available {
                    return None;
                }

                let affixes_amount = prefixes + suffixes;
                let remove_price = forge::affix_operation_price(
                    ForgeAffixOperation::Remove,
                    affixes_amount,
                    &item.item_specs.base,
                )?;
                let add_price = forge::affix_operation_price(
                    ForgeAffixOperation::Add(affix_type),
                    affixes_amount.checked_sub(1)?,
                    &item.item_specs.base,
                )?;

                Some(remove_price + add_price)
            }
            _ => None,
        })
    };

    view! {
        <div class="w-full h-full flex flex-col justify-between relative">
            <CardInsetTitle>"Forge Item"</CardInsetTitle>

            <div class="flex flex-col">
                <span class="text-pink-400 font-bold text-sm xl:text-base">
                    {move || is_equipped().then_some("Equipped Item")}
                </span>
                <ItemDetails selected_item show_affixes=true />
            </div>

            <div class="flex flex-col gap-1 xl:gap-2">
                <div class="w-full flex justify-end">
                    <Checkbox
                        label="Auto confirm remove".to_string()
                        checked=auto_confirm_destructive_actions
                        on_change=move |checked| auto_confirm_destructive_actions.set(checked)
                    />
                </div>
                <div class="flex gap-1 xl:gap-2 w-full mb-1 xl:mb-2">
                    <MenuButton
                        on:click={
                            let try_add_affix = try_add_affix.clone();
                            move |_| try_add_affix(None)
                        }
                        disabled=Signal::derive({
                            move || affix_price().map(|price| price > user_gems()).unwrap_or(true)
                        })
                        class:w-full
                    >
                        <div class="w-full flex justify-center items-center gap-1 text-zinc-400 h-[2em]">
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
                    <MenuButtonRed
                        on:click={
                            let try_replace_affix = try_replace_affix.clone();
                            move |_| try_replace_affix(None)
                        }
                        disabled=Signal::derive({
                            move || {
                                replace_price(None).map(|price| price > user_gems()).unwrap_or(true)
                            }
                        })
                        class:w-full
                    >
                        <div class="w-full flex justify-center items-center gap-1 text-zinc-400 h-[2em]">
                            "Replace random" <span class="text-white font-bold">"Affix"</span>
                            {move || {
                                replace_price(None)
                                    .map(|price| {
                                        view! {
                                            "for "
                                            <span class="text-fuchsia-300 font-bold">{price}</span>
                                            <GemsIcon />
                                        }
                                    })
                            }}
                        </div>
                    </MenuButtonRed>
                </div>
                <div class="flex gap-1 xl:gap-2 w-full">
                    <MenuButton
                        on:click={
                            let try_add_affix = try_add_affix.clone();
                            move |_| try_add_affix(Some(AffixType::Prefix))
                        }
                        disabled=Signal::derive({
                            move || prefix_price().map(|price| price > user_gems()).unwrap_or(true)
                        })
                        class:w-full
                    >
                        <div class="w-full flex justify-center items-center gap-1 text-zinc-400 h-[2em]">
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
                    <MenuButtonRed
                        on:click={
                            let try_replace_affix = try_replace_affix.clone();
                            move |_| try_replace_affix(Some(AffixType::Prefix))
                        }
                        disabled=Signal::derive({
                            move || {
                                replace_price(Some(AffixType::Prefix))
                                    .map(|price| price > user_gems())
                                    .unwrap_or(true)
                            }
                        })
                        class:w-full
                    >
                        <div class="w-full flex justify-center items-center gap-1 text-zinc-400 h-[2em]">
                            "Replace random" <span class="text-white font-bold">"Prefix"</span>
                            {move || {
                                replace_price(Some(AffixType::Prefix))
                                    .map(|price| {
                                        view! {
                                            "for "
                                            <span class="text-fuchsia-300 font-bold">{price}</span>
                                            <GemsIcon />
                                        }
                                    })
                            }}
                        </div>
                    </MenuButtonRed>
                </div>
                <div class="flex gap-1 xl:gap-2 w-full">
                    <MenuButton
                        on:click=move |_| try_add_affix(Some(AffixType::Suffix))
                        disabled=Signal::derive({
                            move || suffix_price().map(|price| price > user_gems()).unwrap_or(true)
                        })
                        class:w-full
                    >
                        <div class="w-full flex justify-center items-center gap-1 text-zinc-400 h-[2em]">
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
                    <MenuButtonRed
                        on:click=move |_| try_replace_affix(Some(AffixType::Suffix))
                        disabled=Signal::derive({
                            move || {
                                replace_price(Some(AffixType::Suffix))
                                    .map(|price| price > user_gems())
                                    .unwrap_or(true)
                            }
                        })
                        class:w-full
                    >
                        <div class="w-full flex justify-center items-center gap-1 text-zinc-400 h-[2em]">
                            "Replace random" <span class="text-white font-bold">"Suffix"</span>
                            {move || {
                                replace_price(Some(AffixType::Suffix))
                                    .map(|price| {
                                        view! {
                                            "for "
                                            <span class="text-fuchsia-300 font-bold">{price}</span>
                                            <GemsIcon />
                                        }
                                    })
                            }}
                        </div>
                    </MenuButtonRed>
                </div>

                <div class="grid grid-cols-2 gap-1 xl:gap-2 w-full mt-1 xl:mt-2">
                    <div></div>
                    <MenuButtonRed
                        on:click=move |_| try_remove_affix()
                        disabled=Signal::derive({
                            move || remove_price().map(|price| price > user_gems()).unwrap_or(true)
                        })
                        class:w-full
                    >
                        <div class="w-full flex justify-center items-center gap-1 text-zinc-400 h-[2em]">
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
                    </MenuButtonRed>
                </div>

            </div>
        </div>
    }
}

#[component]
pub fn SacrificeDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend: BackendClient = expect_context();
    let town_context: TownContext = expect_context();
    let toaster: Toasts = expect_context();
    let discarding = RwSignal::new(false);

    let sacrifice_price = move || {
        selected_item.with(|selected| match selected {
            SelectedItem::InMarket(item)
                if !item
                    .item_specs
                    .base
                    .categories
                    .contains(&ItemCategory::Soul) =>
            {
                Some(computations::sacrifice_price(
                    &item.item_specs.modifiers.affixes,
                ))
            }
            _ => None,
        })
    };

    let soul_selected = move || {
        selected_item.with(|selected| {
            matches!(
                selected,
                SelectedItem::InMarket(item)
                    if item.item_specs.base.categories.contains(&ItemCategory::Soul)
            )
        })
    };

    let do_sacrifice = {
        let character_id = town_context.character.read_untracked().character_id;
        move || {
            if let SelectedItem::InMarket(item) = selected_item.get() {
                spawn_local(async move {
                    match backend
                        .forge_sacrifice(&ForgeSacrificeRequest {
                            character_id,
                            item_index: item.index as u32,
                        })
                        .await
                    {
                        Ok(response) => {
                            let updated_item = response.inventory.bag.get(item.index).cloned();
                            town_context.inventory.set(response.inventory);
                            town_context.character.write().resource_gems = response.resource_gems;
                            if let Some(updated_item) = updated_item {
                                selected_item.set(SelectedItem::InMarket(SelectedMarketItem {
                                    item_specs: Arc::new(updated_item),
                                    ..item
                                }));
                            } else {
                                selected_item.set(SelectedItem::None);
                            }
                        }
                        Err(error) => show_toast(
                            toaster,
                            format!("Failed to sacrifice item: {error}"),
                            ToastVariant::Error,
                        ),
                    }
                });
            }
        }
    };

    let do_discard = {
        let character_id = town_context.character.read_untracked().character_id;
        move || {
            if !discarding.get_untracked()
                && let SelectedItem::InMarket(item) = selected_item.get()
                && item
                    .item_specs
                    .base
                    .categories
                    .contains(&ItemCategory::Soul)
            {
                discarding.set(true);
                spawn_local(async move {
                    match backend
                        .inventory_delete(&InventoryDeleteRequest {
                            character_id,
                            item_indexes: vec![item.index as u8],
                        })
                        .await
                    {
                        Ok(response) => {
                            // Removing an item compacts the bag, so the item that was directly to
                            // its right now occupies the discarded item's index.
                            let next_item = response
                                .inventory
                                .bag
                                .get(item.index)
                                .and_then(|next_item| {
                                    inventory_grid_item(
                                        next_item,
                                        item.index,
                                        InventoryItemEligibility::Any,
                                    )
                                })
                                .map(SelectedItem::InMarket)
                                .unwrap_or(SelectedItem::None);

                            town_context.inventory.set(response.inventory);
                            town_context.character.update(|character| {
                                character.resource_gold = response.resource_gold;
                                character.resource_gems = response.resource_gems;
                            });
                            selected_item.set(next_item);
                        }
                        Err(error) => show_toast(
                            toaster,
                            format!("Failed to discard soul: {error}"),
                            ToastVariant::Error,
                        ),
                    }
                    discarding.set(false);
                });
            }
        }
    };

    // let try_sacrifice = move || {
    //     (confirm_context.confirm)(
    //         "Sacrificing destroys this item and extracts one random affix. Continue?".into(),
    //         Arc::new(do_sacrifice),
    //     );
    // };

    view! {
        <div class="flex h-full w-full flex-col justify-between">
            <CardInsetTitle>
                <span class="inline-flex items-center gap-1">
                    "Sacrifice Item"
                    <HelpTooltip text="Destroy a Rare Item with four affixes to extract one affix as a matching Item Soul." />
                </span>
            </CardInsetTitle>
            <ItemDetails selected_item show_affixes=true />
            <div class="flex gap-1 xl:gap-2 w-full">
                <MenuButton
                    on:click=move |_| do_sacrifice()
                    disabled=Signal::derive(move || {
                        sacrifice_price()
                            .is_none_or(|price| {
                                price > town_context.character.read().resource_gems
                            })
                    })
                    class:w-full
                >
                    <div class="flex w-full items-center justify-center gap-1 text-zinc-400 h-[2em]">
                        {move || {
                            match sacrifice_price() {
                                Some(price) => {
                                    view! {
                                        "Sacrifice for "
                                        <span class="font-bold text-fuchsia-300">{price}</span>
                                        <GemsIcon />
                                    }
                                        .into_any()
                                }
                                None => view! { "Sacrifice Item" }.into_any(),
                            }
                        }}
                    </div>
                </MenuButton>
                <MenuButtonRed
                    on:click=move |_| do_discard()
                    disabled=Signal::derive(move || discarding.get() || !soul_selected())
                    class:w-full
                >
                    "Discard"
                </MenuButtonRed>
            </div>
        </div>
    }
}

#[component]
pub fn ImbueDetails(
    selected_item: RwSignal<SelectedItem>,
    selected_soul: RwSignal<SelectedItem>,
) -> impl IntoView {
    let backend: BackendClient = expect_context();
    let town_context: TownContext = expect_context();
    let toaster: Toasts = expect_context();

    let compatible = move || {
        selected_item.with(|selected_item| {
            selected_soul.with(|selected_soul| match (selected_item, selected_soul) {
                (SelectedItem::InMarket(item), SelectedItem::InMarket(soul)) => {
                    soul.item_specs
                        .base
                        .categories
                        .is_superset(&item.item_specs.base.categories)
                        && soul
                            .item_specs
                            .modifiers
                            .affixes
                            .first()
                            .is_some_and(|affix| {
                                item.item_specs.modifiers.can_replace_soul_affix(affix)
                            })
                }
                _ => false,
            })
        })
    };

    let do_imbue = {
        let character_id = town_context.character.read_untracked().character_id;
        move || {
            if let (SelectedItem::InMarket(item), SelectedItem::InMarket(soul)) =
                (selected_item.get(), selected_soul.get())
            {
                spawn_local(async move {
                    match backend
                        .forge_imbue(&ForgeImbueRequest {
                            character_id,
                            item_index: item.index as u32,
                            soul_index: soul.index as u32,
                        })
                        .await
                    {
                        Ok(response) => {
                            let result_index = if soul.index < item.index {
                                item.index.saturating_sub(1)
                            } else {
                                item.index
                            };
                            let result_item = response.inventory.bag.get(result_index).cloned();

                            town_context.inventory.set(response.inventory);
                            selected_soul.set(SelectedItem::None);
                            if let Some(result_item) = result_item {
                                selected_item.set(SelectedItem::InMarket(SelectedMarketItem {
                                    index: result_index,
                                    item_specs: Arc::new(result_item),
                                    ..item
                                }));
                            } else {
                                selected_item.set(SelectedItem::None);
                            }
                        }
                        Err(error) => show_toast(
                            toaster,
                            format!("Failed to imbue item: {error}"),
                            ToastVariant::Error,
                        ),
                    }
                });
            }
        }
    };

    // let try_imbue = move || {
    //     let replacing = selected_item.with(|selected| match selected {
    //         SelectedItem::InMarket(item) => {
    //             item.item_specs.modifiers.count_affixes(AffixType::Soul) > 0
    //         }
    //         _ => false,
    //     });
    //     let message = if replacing {
    //         "Imbuing consumes the selected Soul and replaces the existing Soul affix. Continue?"
    //     } else {
    //         "Imbuing consumes the selected Soul. Continue?"
    //     };
    //     (confirm_context.confirm)(message.into(), Arc::new(do_imbue));
    // };

    view! {
        <div class="flex h-full w-full min-h-0 flex-col justify-between gap-2">
            <CardInsetTitle>
                <span class="inline-flex items-center gap-1">
                    "Imbue Item"
                    <HelpTooltip text="Imbue a Rare or Soulforged Item using an Item Soul of the same category." />
                </span>
            </CardInsetTitle>
            <div class="flex flex-col gap-2 xl:gap-4 overflow-y-auto">
                <ItemDetails selected_item show_affixes=true />
                <ItemDetails selected_item=selected_soul show_affixes=true compact=true />
            </div>
            <MenuButton on:click=move |_| do_imbue() disabled=Signal::derive(move || !compatible())>
                {move || {
                    if compatible() { "Imbue Item" } else { "Select a compatible Item and Soul" }
                }}
            </MenuButton>
        </div>
    }
}

#[component]
pub fn UpgradeUniqueDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend: BackendClient = expect_context();
    let town_context: TownContext = expect_context();
    let toaster: Toasts = expect_context();
    let confirm_context: ConfirmContext = expect_context();

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
                            .forge_upgrade(&ForgeUpgradeRequest {
                                character_id,
                                item_index: item.index as u32,
                            })
                            .await
                        {
                            Ok(response) => {
                                let updated_item_specs =
                                    response.inventory.bag.get(item.index).cloned();

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

    let try_upgrade_item = {
        let confirm_context = confirm_context.clone();
        move || {
            let do_upgrade_item = Arc::new(do_upgrade_item);
            let next_power_level = selected_item.with(|selected_item| match selected_item {
                SelectedItem::InMarket(item) => item
                    .item_specs
                    .base
                    .upgrade_levels
                    .get(item.item_specs.modifiers.upgrade_level as usize)
                    .map(|next_upgrade_level| {
                        item.item_specs.required_level.max(*next_upgrade_level)
                    }),
                _ => None,
            });
            let character_power_level = town_context.character.read_untracked().max_area_level;

            if let Some(next_power_level) = next_power_level
                && next_power_level > character_power_level
            {
                (confirm_context.confirm)(
                    format!(
                        "Empowering this item will raise its required Power Level to {next_power_level}, above your Character Power Level of {character_power_level}, and make it unusable. Continue?"
                    ),
                    do_upgrade_item,
                );
            } else {
                do_upgrade_item();
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
            <CardInsetTitle>
                <span class="inline-flex items-center gap-1">
                    "Empower Unique Item"
                    <HelpTooltip text="Empower upgrades a low-level Unique Item into a stronger version of itself. The maximum Empower level is determined by the Item Level." />
                </span>
            </CardInsetTitle>

            <div class="flex flex-col">
                <span class="text-pink-400 font-bold text-sm xl:text-base">
                    {move || is_equipped().then_some("Equipped Item")}
                </span>
                <ItemDetails selected_item show_affixes=true />
            </div>

            <div class="flex flex-col gap-1 xl:gap-2">
                <MenuButton
                    on:click=move |_| try_upgrade_item()
                    disabled=Signal::derive({
                        move || upgrade_price().map(|price| price > user_gems()).unwrap_or(true)
                    })
                >
                    // class:mb-1
                    // class:xl:mb-2
                    <div class="w-full flex justify-center items-center gap-1 text-zinc-400 h-[2em]">
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
                        .gamble_item(&GambleItemRequest {
                            character_id,
                            item_category: gamble_category.get(),
                        })
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
                            .inventory_delete(&InventoryDeleteRequest {
                                character_id,
                                item_indexes: [selected_item.index as u8].into(),
                            })
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
            <CardInsetTitle>"Gamble"</CardInsetTitle>

            <div class="flex flex-col w-full">
                // <div class="w-full flex items-center justify-center text-sm xl:text-base text-amber-400 font-bold gap-2">
                // {move || { loot_filter_category_to_str(gamble_category.get()) }}
                // </div>
                <div class="w-full p-2 gap-2 flex items-center justify-center text-zinc-400 text-sm">
                    <span>"Item Category:"</span>
                    <SearchableDropdownMenu
                        options=GAMBLE_ITEM_CATEGORIES
                            .iter()
                            .map(|item_category| (
                                *item_category,
                                loot_filter_category_to_str(*item_category).to_string(),
                            ))
                            .collect()
                        chosen_option=gamble_category
                    />
                </div>
                <ItemDetails selected_item show_affixes=true />
            </div>

            <div class="flex flex-col gap-2 w-full">
                <Separator />
                <div class="w-full flex items-center gap-1 text-lg text-zinc-400 p-2">
                    "Price: "
                    <span class="text-fuchsia-300 font-bold">
                        {move || format!("{:.0}", gamble_price())}
                    </span> <GemsIcon />
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
                            move || {
                                gamble_price() > user_gems() || selected_item.read().is_empty()
                            }
                        })
                        class:w-full
                    >
                        "Discard & Retry"
                    </MenuButtonRed>
                </div>
            </div>
        </div>
    }
}
