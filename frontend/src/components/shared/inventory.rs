use std::{collections::HashSet, sync::Arc, time::Duration};

use leptos::{html::Div, portal::Portal, prelude::*, web_sys};

use shared::data::{
    area::AreaLevel,
    item::{InventorySortType, ItemCategory, ItemRarity, ItemSlot, ItemSpecs},
    player::{EquippedSlot, PlayerInventory},
};

use crate::{
    assets::img_asset,
    components::{
        chat::chat_context::ChatContext,
        events::{EventsContext, Key},
        shared::{
            bag_grid::{BagAction, BagConfig, BagGrid, BagSortButton, EmptySlot},
            item_card::ItemCard,
            resources::{ResourceReward, ResourceRewardOverlay},
            tooltips::ItemTooltip,
        },
        ui::{
            buttons::{CloseButton, MenuButton},
            card::{CardInset, CardTitle, MenuCard},
            context_menu::{ActionMenuRow, ActionMenuTone, ContextMenu},
            menu_panel::MenuPanel,
            tooltip::DynamicTooltipPosition,
        },
    },
};

type SellQueue = RwSignal<HashSet<usize>>;

#[derive(Clone, Default)]
pub enum InventoryEquipFilter {
    #[default]
    Slot,
    Map(String),
    Rune,
    Rarity {
        // Maybe later rename to Any, and not_item_rarity
        item_rarity: ItemRarity,
        not: bool,
    },
    Bag,
}

#[derive(Clone, Default)]
pub struct InventoryConfig {
    pub player_inventory: RwSignal<PlayerInventory>,
    // pub loot_preference: Option<RwSignal<Option<ItemCategory>>>,
    pub on_loot_filter: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_unequip: Option<Arc<dyn Fn(ItemSlot) + Send + Sync>>,
    pub on_sheathe: Option<Arc<dyn Fn(ItemSlot) + Send + Sync>>,
    pub on_equip: Option<Arc<dyn Fn(u8) + Send + Sync>>,
    pub on_sell: Option<Arc<dyn Fn(Vec<u8>) + Send + Sync>>,
    pub on_sort: Option<Arc<dyn Fn(InventorySortType) + Send + Sync>>,
    pub sell_reward: RwSignal<ResourceReward>,
    pub max_item_level: Signal<AreaLevel>,
    pub equip_filter: Signal<InventoryEquipFilter>,
}

#[component]
pub fn Inventory(inventory: InventoryConfig, open: RwSignal<bool>) -> impl IntoView {
    let sell_queue = SellQueue::default();
    provide_context(sell_queue);

    Effect::new(move || {
        if !open.get() {
            sell_queue.write().drain();
        }
    });

    view! {
        <MenuPanel open=open h_full=false center=false>
            <div class="relative w-full max-h-full flex justify-between gap-1 xl:gap-4 ">
                <EquippedItemsCard inventory=inventory.clone() class:justify-self-end />
                <BagCard inventory=inventory.clone() open=open class:justify-self-start />
            </div>
        </MenuPanel>
    }
}

#[component]
pub fn EquippedItemsCard(inventory: InventoryConfig) -> impl IntoView {
    const EQUIPPED_SLOTS: &[(ItemSlot, &str, &str)] = &[
        (ItemSlot::Accessory, "ui/accessory.webp", "Accessory"),
        (ItemSlot::Helmet, "ui/helmet.webp", "Helmet"),
        (ItemSlot::Amulet, "ui/amulet.webp", "Amulet"),
        (ItemSlot::Weapon, "ui/weapon.webp", "Weapon"),
        (ItemSlot::Body, "ui/shirt.webp", "Body Armor"),
        (ItemSlot::Shield, "ui/shield.webp", "Shield"),
        (ItemSlot::Gloves, "ui/gloves.webp", "Gloves"),
        (ItemSlot::Boots, "ui/boots.webp", "Boots"),
        (ItemSlot::Ring, "ui/ring.webp", "Ring"),
    ];

    view! {
        // <div class="w-[30%] h-full flex flex-col gap-1 xl:gap-2 p-1 xl:p-2 bg-zinc-800 rounded-md shadow-xl ring-1 ring-zinc-950">
        <MenuCard class="w-[30%] h-full">

            // <p class="text-shadow-md shadow-gray-950 text-amber-200 text-l xl:text-xl">
            // <span class="font-bold">"Equipped"</span>
            // </p>
            <CardTitle>"Equipped"</CardTitle>

            // <div class="relative min-h-0 flex-1  overflow-y-auto">
            <CardInset class="relative min-h-0 flex-1">
                <div class="grid grid-rows-3 grid-cols-3 gap-2 xl:gap-x-4 xl:gap-y-3 px-2 xl:px-3">
                    {EQUIPPED_SLOTS
                        .iter()
                        .map(|(slot, asset, alt)| {
                            view! {
                                <EquippedItem
                                    inventory=inventory.clone()
                                    item_slot=*slot
                                    fallback_asset=*asset
                                    fallback_alt=*alt
                                />
                            }
                        })
                        .collect::<Vec<_>>()}
                </div>
            </CardInset>
        </MenuCard>
    }
}

#[component]
fn EquippedItem(
    inventory: InventoryConfig,
    item_slot: ItemSlot,
    fallback_asset: &'static str,
    fallback_alt: &'static str,
) -> impl IntoView {
    let show_menu = RwSignal::new(false);

    let render_fallback = move || {
        view! {
            <EmptySlot>
                <img
                    draggable="false"
                    src=img_asset(fallback_asset)
                    alt=fallback_alt
                    class="object-contain max-w-full max-h-full opacity-20"
                />
            </EmptySlot>
        }
        .into_any()
    };

    let equipped_item = move || {
        inventory
            .player_inventory
            .read()
            .equipped
            .get(&item_slot)
            .cloned()
    };

    view! {
        <div class="relative group w-full aspect-[2/3]">
            {move || match equipped_item() {
                Some(EquippedSlot::MainSlot(item_specs)) => {
                    let item_specs = Arc::new(*item_specs.clone());
                    view! {
                        <EquippedItemEquippedSlot
                            inventory=inventory.clone()
                            item_slot
                            item_specs
                            show_menu
                        />
                    }
                        .into_any()
                }
                Some(EquippedSlot::ExtraSlot(main_slot)) => {
                    if let Some(EquippedSlot::MainSlot(item_specs)) = inventory
                        .player_inventory
                        .read()
                        .equipped
                        .get(&main_slot)
                        .cloned()
                    {
                        view! {
                            <EmptySlot>
                                <img
                                    draggable="false"
                                    src=img_asset(&item_specs.base.icon)
                                    alt=fallback_alt
                                    class="object-contain max-w-full max-h-full opacity-50"
                                />
                            </EmptySlot>
                        }
                            .into_any()
                    } else {
                        render_fallback()
                    }
                }
                None => render_fallback(),
            }}
        </div>
    }
}

#[component]
fn EquippedItemEquippedSlot(
    inventory: InventoryConfig,
    item_slot: ItemSlot,
    item_specs: Arc<ItemSpecs>,
    show_menu: RwSignal<bool>,
) -> impl IntoView {
    let item_ref = NodeRef::new();
    let chat_context: ChatContext = expect_context();
    let events_context: EventsContext = expect_context();

    let equipped_item_rarity = item_specs.modifiers.rarity;
    let is_weapon = item_specs.weapon_specs.is_some();
    let is_sheathed = Signal::derive({
        let inventory = inventory.clone();
        move || {
            inventory
                .player_inventory
                .read()
                .sheathed
                .contains(&item_slot)
        }
    });
    let can_unequip = Signal::derive(move || {
        inventory
            .equip_filter
            .with(|equip_filter| match equip_filter {
                InventoryEquipFilter::Slot => true,
                InventoryEquipFilter::Map(_)
                | InventoryEquipFilter::Rune
                | InventoryEquipFilter::Bag => false,
                InventoryEquipFilter::Rarity { item_rarity, not } => {
                    (equipped_item_rarity == *item_rarity) != *not
                }
            })
    });

    let is_being_unequipped = RwSignal::new(false);
    view! {
        <div
            node_ref=item_ref
            class="relative w-full h-full overflow-visible"
            class:brightness-50=move || !can_unequip.get()
        >
            <ItemCard
                item_specs=item_specs.clone()
                on:click={
                    let item_specs = item_specs.clone();
                    move |_| {
                        if events_context.key_pressed(Key::Shift) {
                            chat_context.link_item(item_specs.clone());
                        } else {
                            show_menu.set(true);
                        }
                    }
                }
                tooltip_position=DynamicTooltipPosition::Auto
                max_item_level=inventory.max_item_level
            />

            <Show when=move || is_being_unequipped.get()>
                <div
                    class="absolute inset-0 z-30 w-full"
                    style="
                    background:
                    linear-gradient(180deg, rgba(214,177,102,0.04), rgba(0,0,0,0.08)),
                    linear-gradient(135deg, rgba(32,31,36,0.82), rgba(8,8,10,0.92));
                    box-shadow: inset 0 0 0 1px rgba(108,83,41,0.55), inset 0 0 18px rgba(0,0,0,0.45);"
                ></div>
            </Show>

            <Show when=move || show_menu.get()>
                <EquippedItemContextMenu
                    inventory=inventory.clone()
                    item_slot=item_slot
                    is_being_unequipped=is_being_unequipped
                    on_close=Callback::new(move |_| show_menu.set(false))
                    can_unequip
                    is_weapon
                    is_sheathed
                />
                {
                    let item_specs = item_specs.clone();
                    view! {
                        <Portal>
                            {
                                let tooltip_ref = NodeRef::new();
                                let tooltip_size = Memo::new(move |_| {
                                    let tooltip_div: Option<web_sys::HtmlDivElement> = tooltip_ref
                                        .get();
                                    tooltip_div
                                        .map(|tooltip_div| {
                                            let rect = tooltip_div.get_bounding_client_rect();
                                            (rect.width(), rect.height())
                                        })
                                        .unwrap_or_default()
                                });
                                let tooltip_pos = move || {
                                    let item_div: web_sys::HtmlDivElement = item_ref.get().unwrap();
                                    let item_rect = item_div.get_bounding_client_rect();
                                    let (tooltip_width, tooltip_height) = tooltip_size.get();
                                    let window_height = web_sys::window()
                                        .unwrap()
                                        .inner_height()
                                        .unwrap()
                                        .as_f64()
                                        .unwrap();
                                    let window_width = web_sys::window()
                                        .unwrap()
                                        .inner_width()
                                        .unwrap()
                                        .as_f64()
                                        .unwrap();
                                    (
                                        item_rect.right().min(window_width - tooltip_width),
                                        item_rect.top().min(window_height - tooltip_height),
                                    )
                                };

                                view! {
                                    <div
                                        node_ref=tooltip_ref
                                        class="fixed  z-50 transition-opacity duration-150 text-center px-2"
                                        style=move || {
                                            let (x, y) = tooltip_pos();
                                            format!("left:{}px; top:{}px;", x, y)
                                        }
                                    >
                                        <ItemTooltip
                                            item_specs=item_specs.clone()
                                            max_item_level=inventory.max_item_level
                                        />
                                    </div>
                                }
                            }
                        </Portal>
                    }
                }
            </Show>
        </div>
    }
}

#[component]
pub fn EquippedItemContextMenu(
    inventory: InventoryConfig,
    item_slot: ItemSlot,
    on_close: Callback<()>,
    is_being_unequipped: RwSignal<bool>,
    can_unequip: Signal<bool>,
    is_weapon: bool,
    is_sheathed: Signal<bool>,
) -> impl IntoView {
    view! {
        <ContextMenu on_close=on_close>
            {inventory
                .on_sheathe
                .clone()
                .and_then(|on_sheathe| {
                    is_weapon
                        .then(|| {
                            view! {
                                <ActionMenuRow
                                    label_signal=Signal::derive(move || {
                                        if is_sheathed.get() {
                                            "Unsheathe".to_string()
                                        } else {
                                            "Sheathe".to_string()
                                        }
                                    })
                                    tone=ActionMenuTone::Skill
                                    on_click=move || {
                                        on_sheathe(item_slot);
                                        on_close.run(());
                                    }
                                />
                            }
                        })
                })}
            {inventory
                .on_unequip
                .map(|on_unequip| {
                    can_unequip
                        .get_untracked()
                        .then(|| {

                            view! {
                                <ActionMenuRow
                                    label=if let InventoryEquipFilter::Slot = inventory
                                        .equip_filter
                                        .get_untracked()
                                    {
                                        "Unequip"
                                    } else {
                                        "Use"
                                    }
                                    tone=ActionMenuTone::Success
                                    on_click=move || {
                                        on_unequip(item_slot);
                                        on_close.run(());
                                        is_being_unequipped.set(true);
                                        set_timeout(
                                            move || is_being_unequipped.set(false),
                                            Duration::from_millis(1000),
                                        );
                                    }
                                />
                            }
                        })
                })}
            <ActionMenuRow
                label="Cancel"
                tone=ActionMenuTone::Neutral
                on_click=move || on_close.run(())
            />
        </ContextMenu>
    }
}

/// Inventory-specific comparison supplied to any item grid displaying these items.
pub fn equipped_item_comparison(
    inventory: RwSignal<PlayerInventory>,
) -> Callback<ItemSlot, Option<Arc<ItemSpecs>>> {
    Callback::new(move |slot| {
        inventory.with(|inventory| match inventory.equipped.get(&slot) {
            Some(EquippedSlot::MainSlot(item)) => Some(Arc::from(item.clone())),
            _ => None,
        })
    })
}

fn inventory_bag_config(inventory: &InventoryConfig, sell_queue: SellQueue) -> BagConfig {
    let player_inventory = inventory.player_inventory;
    let equip_filter = inventory.equip_filter;
    let items = Memo::new(move |_| player_inventory.read().bag.clone());
    let can_equip = Callback::new(move |index: usize| {
        items.with(|items| {
            items.get(index).is_some_and(|item| {
                equip_filter.with(|filter| match filter {
                    InventoryEquipFilter::Slot => item.base.slot.is_some(),
                    InventoryEquipFilter::Map(area_id) => {
                        item.base.map_specs.as_ref().is_some_and(|map| {
                            map.area_id
                                .as_ref()
                                .is_none_or(|map_area| area_id == map_area)
                        })
                    }
                    InventoryEquipFilter::Rune => item.base.rune_specs.is_some(),
                    InventoryEquipFilter::Rarity { item_rarity, not } => {
                        (item.modifiers.rarity == *item_rarity) != *not
                    }
                    InventoryEquipFilter::Bag => true,
                })
            })
        })
    });
    let mut actions = Vec::new();
    if let Some(on_equip) = inventory.on_equip.clone() {
        actions.push(BagAction {
            label: Callback::new(move |_| {
                if matches!(equip_filter.get(), InventoryEquipFilter::Slot) {
                    "Equip"
                } else {
                    "Use"
                }
                .into()
            }),
            visible: Some(can_equip),
            on_action: Callback::new(move |index| {
                on_equip(index as u8);
                sell_queue.write().remove(&index);
            }),
            pending_duration: Some(Duration::from_millis(1000)),
            ..Default::default()
        });
    }
    if inventory.on_sell.is_some() {
        actions.push(BagAction {
            label: Callback::new(move |index| {
                if sell_queue.read().contains(&index) {
                    "Unsell"
                } else {
                    "Sell"
                }
                .into()
            }),
            tone: ActionMenuTone::Warning,
            right_click: true,
            on_action: Callback::new(move |index| {
                sell_queue.update(|queue| {
                    if !queue.remove(&index) {
                        queue.insert(index);
                    }
                })
            }),
            ..Default::default()
        });
    }
    BagConfig {
        items: items.into(),
        capacity: Signal::derive(move || player_inventory.read().max_bag_size as usize),
        max_item_level: inventory.max_item_level,
        comparable_item: Some(equipped_item_comparison(player_inventory)),
        dimmed: Some(Callback::new(move |index| !can_equip.run(index))),
        sell_badge: Some(Callback::new(move |index| {
            sell_queue.read().contains(&index)
        })),
        show_sell_price: inventory.on_sell.is_some(),
        actions,
        ..Default::default()
    }
}

#[component]
fn BagCard(inventory: InventoryConfig, open: RwSignal<bool>) -> impl IntoView {
    let sell_queue: SellQueue = expect_context();
    let bag = inventory_bag_config(&inventory, sell_queue);

    view! {
        <MenuCard class="h-full w-[70%]">
            <div class="px-4 relative z-10 flex items-center justify-between gap-2">
                <div class="flex flex-row items-center gap-1 xl:gap-2">
                    <CardTitle>"Inventory"</CardTitle>
                    <span class="text-shadow-md shadow-gray-950 text-zinc-400 text-xs xl:text-base font-medium">
                        {move || {
                            format!(
                                "({} / {})",
                                inventory.player_inventory.read().bag.len(),
                                inventory.player_inventory.read().max_bag_size,
                            )
                        }}
                    </span>
                    {inventory
                        .on_sort
                        .clone()
                        .map(|on_sort| {
                            view! {
                                <BagSortButton on_sort=move |sort_type| {
                                    sell_queue.write().drain();
                                    on_sort(sort_type);
                                } />
                            }
                        })}
                </div>

                {
                    let on_loot_filter = inventory.on_loot_filter.clone();
                    on_loot_filter
                        .map(|on_loot_filter| {

                            view! {
                                <MenuButton on:click=move |_| on_loot_filter()>
                                    "Loot Filter"
                                </MenuButton>
                            }
                        })
                }

                <div class="flex items-center gap-1 xl:gap-2">
                    <SellAllButton inventory=inventory.clone() />
                    <CloseButton on:click=move |_| open.set(false) />
                </div>
            </div>

            <CardInset class="relative min-h-0 flex-1">
                <BagGrid config=bag.clone() />
            </CardInset>

        </MenuCard>
    }
}

#[component]
fn SellAllButton(inventory: InventoryConfig) -> impl IntoView {
    inventory.on_sell.map(|on_sell| {
        let button_ref = NodeRef::<Div>::new();
        let disabled = Signal::derive({
            let sell_queue = expect_context::<SellQueue>();
            move || sell_queue.read().is_empty()
        });
        let reward_position = move || {
            let _ = inventory.sell_reward.get();
            button_ref
                .get()
                .map(|button| {
                    let rect = button.get_bounding_client_rect();
                    format!(
                        "left:{}px; top:{}px; width:{}px; height:{}px;",
                        rect.left(),
                        rect.top(),
                        rect.width(),
                        rect.height(),
                    )
                })
                .unwrap_or_else(|| "display:none;".to_string())
        };
        view! {
            <div node_ref=button_ref class="relative overflow-visible">
                <MenuButton
                    on:click={
                        let sell_queue = expect_context::<SellQueue>();
                        move |_| { on_sell(sell_queue.write().drain().map(|x| x as u8).collect()) }
                    }
                    disabled=disabled
                >
                    <span class="inline xl:hidden">"Sell all"</span>
                    <span class="hidden xl:inline font-variant:small-caps">
                        "Sell all marked items"
                    </span>
                </MenuButton>
                <Portal>
                    <div
                        class="fixed z-50 pointer-events-none overflow-visible"
                        style=reward_position
                    >
                        <ResourceRewardOverlay reward=inventory.sell_reward />
                    </div>
                </Portal>
            </div>
        }
    })
}

pub fn loot_filter_category_to_str(opt: Option<ItemCategory>) -> &'static str {
    use ItemCategory::*;
    match opt {
        Some(item_category) => match item_category {
            Armor => "Any Armor",
            Jewelry => "Any Jewelry",
            Accessory => "Cloak",
            AttackWeapon => "Attack Weapon",
            SpellWeapon => "Spell Weapon",
            MeleeWeapon => "Melee Weapon",
            RangedWeapon => "Ranged Weapon",
            MeleeWeapon1H => "One-Handed Melee Weapon",
            MeleeWeapon2H => "Two-Handed Melee Weapon",
            Shield => "Shield",
            Focus => "Magical Focus",
            Amulet => "Amulet",
            Body => "Body Armor",
            Boots => "Boots",
            Cloak => "Cloak",
            Gloves => "Gloves",
            Helmet => "Helmet",
            Ring => "Ring",
            Map => "Edict",
            Rune => "Rune",
        },
        None => "Any Item",
    }
}
