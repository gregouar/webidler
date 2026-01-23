use chrono::Utc;
use leptos::{prelude::*, task::spawn_local};
use shared::{
    data::{forge, item::ItemRarity, item_affix::AffixType, player::EquippedSlot},
    http::client::ForgeAddAffixRequest,
};
use std::sync::Arc;

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    shared::resources::GemsIcon,
    town::{
        items_browser::{ItemDetails, ItemsBrowser, SelectedItem, SelectedMarketItem},
        TownContext,
    },
    ui::{
        buttons::MenuButton,
        card::{Card, CardHeader, CardInset, CardTitle},
        menu_panel::MenuPanel,
        toast::*,
    },
};

#[component]
pub fn ForgePanel(open: RwSignal<bool>) -> impl IntoView {
    let selected_item = RwSignal::new(SelectedItem::None);

    view! {
        <MenuPanel open=open>
            <Card class="h-full">
                <CardHeader title="Forge" on_close=move || open.set(false) />

                <div class="grid grid-cols-2 gap-2 min-h-0 flex-1">
                    <CardInset class="w-full" pad=false>
                        <InventoryBrowser selected_item />
                    </CardInset>

                    <CardInset class="w-full">
                        <ForgeDetails selected_item />
                    </CardInset>
                </div>
            </Card>
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
pub fn ForgeDetails(selected_item: RwSignal<SelectedItem>) -> impl IntoView {
    let backend = expect_context::<BackendClient>();
    let town_context = expect_context::<TownContext>();
    let auth_context = expect_context::<AuthContext>();
    let toaster = expect_context::<Toasts>();

    let user_gems = move || town_context.character.read().resource_gems;

    let do_add_affix = {
        let character_id = town_context.character.read_untracked().character_id;
        move |affix_type| {
            if let SelectedItem::InMarket(item) = selected_item.get() {
                spawn_local({
                    async move {
                        match backend
                            .forge_add_affix(
                                &auth_context.token(),
                                &ForgeAddAffixRequest {
                                    character_id,
                                    item_index: item.index as u32,
                                    affix_type,
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
                    on:click=move |_| do_add_affix(None)
                    disabled=Signal::derive({
                        move || affix_price().map(|price| price > user_gems()).unwrap_or(true)
                    })
                    class:mb-1
                    class:xl:mb-2
                >
                    <div class="w-full flex justify-center items-center gap-1 text-gray-400 h-[2em]">
                        "Add" <span class="text-white font-bold">"Affix"</span>
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
                    on:click=move |_| do_add_affix(Some(AffixType::Prefix))
                    disabled=Signal::derive({
                        move || prefix_price().map(|price| price > user_gems()).unwrap_or(true)
                    })
                >
                    <div class="w-full flex justify-center items-center gap-1 text-gray-400 h-[2em]">
                        "Add" <span class="text-white font-bold">"Prefix"</span>
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
                    on:click=move |_| do_add_affix(Some(AffixType::Suffix))
                    disabled=Signal::derive({
                        move || suffix_price().map(|price| price > user_gems()).unwrap_or(true)
                    })
                >
                    <div class="w-full flex justify-center items-center gap-1 text-gray-400 h-[2em]">
                        "Add" <span class="text-white font-bold">"Suffix"</span>
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

            </div>
        </div>
    }
}
