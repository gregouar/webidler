use leptos::{prelude::*, task::spawn_local};
use leptos_toaster::Toasts;
use std::sync::Arc;

use shared::http::client::{
    InventoryDeleteRequest, InventoryEquipRequest, InventoryUnequipRequest,
};

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    shared::{
        inventory::{Inventory, InventoryConfig, InventoryEquipFilter, SellType},
        loot_filter::LootFilterPanel,
    },
    town::TownContext,
    ui::toast::*,
};

#[component]
pub fn TownInventoryPanel(
    open: RwSignal<bool>,
    #[prop(default = false)] view_only: bool,
) -> impl IntoView {
    let auth_context = expect_context::<AuthContext>();
    let backend = expect_context::<BackendClient>();
    let toaster = expect_context::<Toasts>();
    let town_context = expect_context::<TownContext>();

    let open_loot_filter = RwSignal::new(false);

    let on_equip = move |item_index| {
        let character_id = town_context.character.read_untracked().character_id;
        town_context
            .equip_filter
            .with(|equip_filter| match equip_filter {
                InventoryEquipFilter::Slot => spawn_local({
                    async move {
                        match backend
                            .inventory_equip(
                                &auth_context.token(),
                                &InventoryEquipRequest {
                                    character_id,
                                    item_index,
                                },
                            )
                            .await
                        {
                            Ok(response) => town_context.inventory.set(response.inventory),
                            Err(e) => show_toast(
                                toaster,
                                format!("Failed to equip item: {e}"),
                                ToastVariant::Error,
                            ),
                        }
                    }
                }),
                InventoryEquipFilter::Map(_) | InventoryEquipFilter::Rune => {
                    town_context.selected_item_index.set(Some(item_index));
                    town_context.open_inventory.set(false);
                }
            })
    };

    let on_unequip = move |item_slot| {
        let character_id = town_context.character.read_untracked().character_id;
        spawn_local({
            async move {
                match backend
                    .inventory_unequip(
                        &auth_context.token(),
                        &InventoryUnequipRequest {
                            character_id,
                            item_slot,
                        },
                    )
                    .await
                {
                    Ok(response) => town_context.inventory.set(response.inventory),
                    Err(e) => show_toast(
                        toaster,
                        format!("Failed to unequip item: {e}"),
                        ToastVariant::Error,
                    ),
                }
            }
        })
    };

    let on_sell = move |item_indexes: Vec<u8>| {
        let character_id = town_context.character.read_untracked().character_id;

        spawn_local({
            async move {
                match backend
                    .inventory_delete(
                        &auth_context.token(),
                        &InventoryDeleteRequest {
                            character_id,
                            item_indexes: item_indexes.clone(),
                        },
                    )
                    .await
                {
                    Ok(response) => {
                        town_context
                            .selected_item_index
                            .update(|selected_item_index| {
                                if let Some(selected_item_index) = selected_item_index {
                                    *selected_item_index = selected_item_index.saturating_sub(
                                        item_indexes
                                            .into_iter()
                                            .filter(|index| *index < *selected_item_index)
                                            .count() as u8,
                                    );
                                }
                            });

                        town_context.inventory.set(response.inventory);
                    }
                    Err(e) => show_toast(
                        toaster,
                        format!("Failed to discard items: {e}"),
                        ToastVariant::Error,
                    ),
                }
            }
        })
    };

    let inventory_config = if view_only {
        InventoryConfig {
            player_inventory: town_context.inventory,
            ..Default::default()
        }
    } else {
        InventoryConfig {
            player_inventory: town_context.inventory,
            // loot_preference: None,
            on_loot_filter: Some(Arc::new(move || open_loot_filter.set(true))),
            on_unequip: Some(Arc::new(on_unequip)),
            on_equip: Some(Arc::new(on_equip)),
            on_sell: Some(Arc::new(on_sell)),
            sell_type: SellType::Discard,
            max_item_level: Signal::derive(move || town_context.character.read().max_area_level),
            equip_filter: town_context.equip_filter.into(),
        }
    };

    Effect::new(move || {
        if !open.get() {
            open_loot_filter.set(false);
        }
    });

    let loot_filter = RwSignal::new(Default::default());

    view! {
        <Inventory open=open inventory=inventory_config />
        <LootFilterPanel
            open=open_loot_filter
            loot_filter=loot_filter
            character_id=town_context.character.read_untracked().character_id
            character_name=town_context.character.read_untracked().name.clone()
        />
    }
}
