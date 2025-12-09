use leptos::{prelude::*, task::spawn_local};
use leptos_toaster::Toasts;
use std::sync::Arc;

use shared::http::client::{
    InventoryDeleteRequest, InventoryEquipRequest, InventoryUnequipRequest,
};

use crate::components::{
    auth::AuthContext,
    backend_client::BackendClient,
    shared::inventory::{Inventory, InventoryConfig, SellType},
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

    let on_equip = move |item_index| {
        let character_id = town_context.character.read_untracked().character_id;
        spawn_local({
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

    let on_sell = move |item_indexes| {
        let character_id = town_context.character.read_untracked().character_id;
        spawn_local({
            async move {
                match backend
                    .inventory_delete(
                        &auth_context.token(),
                        &InventoryDeleteRequest {
                            character_id,
                            item_indexes,
                        },
                    )
                    .await
                {
                    Ok(response) => town_context.inventory.set(response.inventory),
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
            loot_preference: None,
            on_unequip: Some(Arc::new(on_unequip)),
            on_equip: Some(Arc::new(on_equip)),
            on_sell: Some(Arc::new(on_sell)),
            sell_type: SellType::Discard,
            max_item_level: Signal::derive(move || town_context.character.read().max_area_level),
        }
    };

    view! { <Inventory open=open inventory=inventory_config /> }
}
