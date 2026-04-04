use std::{collections::HashSet, sync::Arc, time::Duration};

use leptos::{portal::Portal, prelude::*, web_sys};
use leptos_use::on_click_outside;

use shared::data::{
    area::AreaLevel,
    item::{ItemCategory, ItemSlot, ItemSpecs},
    player::{EquippedSlot, PlayerInventory},
};

use crate::{
    assets::img_asset,
    components::{
        accessibility::AccessibilityContext,
        chat::chat_context::ChatContext,
        events::{EventsContext, Key},
        shared::{item_card::ItemCard, tooltips::ItemTooltip},
        ui::{
            buttons::{CloseButton, MenuButton},
            card::{CardInset, CardTitle, MenuCard},
            menu_panel::MenuPanel,
            tooltip::DynamicTooltipPosition,
        },
    },
};

type SellQueue = RwSignal<HashSet<usize>>;

#[derive(Clone, Copy, Default)]
pub enum SellType {
    #[default]
    Sell,
    Discard,
}

#[derive(Clone, Default)]
pub enum InventoryEquipFilter {
    #[default]
    Slot,
    Map(String),
    Rune,
}

#[derive(Clone, Default)]
pub struct InventoryConfig {
    pub player_inventory: RwSignal<PlayerInventory>,
    // pub loot_preference: Option<RwSignal<Option<ItemCategory>>>,
    pub on_loot_filter: Option<Arc<dyn Fn() + Send + Sync>>,
    pub on_unequip: Option<Arc<dyn Fn(ItemSlot) + Send + Sync>>,
    pub on_equip: Option<Arc<dyn Fn(u8) + Send + Sync>>,
    pub on_sell: Option<Arc<dyn Fn(Vec<u8>) + Send + Sync>>,
    pub sell_type: SellType,
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

    let is_being_unequipped = RwSignal::new(false);
    view! {
        <div node_ref=item_ref class="relative w-full h-full overflow-visible">
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
                    style="clip-path: polygon(8px 0, calc(100% - 8px) 0, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 0 calc(100% - 8px), 0 8px);
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
) -> impl IntoView {
    let (success_text, success_border, success_sheen) =
        action_menu_button_tone(ActionMenuTone::Success);
    let (neutral_text, neutral_border, neutral_sheen) =
        action_menu_button_tone(ActionMenuTone::Neutral);

    view! {
        <ContextMenu on_close=on_close>
            {inventory
                .on_unequip
                .map(|on_unequip| {
                    view! {
                        <button
                            class=format!("{} {}", action_menu_button_class(), success_text)
                            style=action_menu_button_style(success_border, success_sheen)
                            on:click=move |_| {
                                on_unequip(item_slot);
                                on_close.run(());
                                is_being_unequipped.set(true);
                                set_timeout(
                                    move || is_being_unequipped.set(false),
                                    Duration::from_millis(1000),
                                );
                            }
                        >
                            "Unequip"
                        </button>
                    }
                })}
            <button
                class=format!("{} {}", action_menu_button_class(), neutral_text)
                style=action_menu_button_style(neutral_border, neutral_sheen)
                on:click=move |_| on_close.run(())
            >
                "Cancel"
            </button>
        </ContextMenu>
    }
}

#[component]
fn BagCard(inventory: InventoryConfig, open: RwSignal<bool>) -> impl IntoView {
    view! {
        // <div class="bg-zinc-800 rounded-md h-full w-[70%] gap-1 xl:gap-2 p-1 xl:p-2 shadow-lg ring-1 ring-zinc-950 relative flex flex-col">
        <MenuCard class="h-full w-[70%]">
            <div class="px-4 relative z-10 flex items-center justify-between gap-2">
                <div class="flex flex-row items-center gap-1 xl:gap-2">
                    <CardTitle>"Inventory"</CardTitle>
                    <span class="text-shadow-md shadow-gray-950 text-gray-400 text-xs xl:text-base font-medium">
                        {move || {
                            format!(
                                "({} / {})",
                                inventory.player_inventory.read().bag.len(),
                                inventory.player_inventory.read().max_bag_size,
                            )
                        }}
                    </span>
                </div>

                // {inventory
                // .loot_preference
                // .map(|loot_preference| {
                // view! {
                // <div class="flex items-center gap-2">
                // <span class="hidden xl:inline text-gray-400 text-sm">
                // "Loot Preference:"
                // </span>
                // <LootFilterDropdown loot_preference />
                // </div>
                // }
                // })}

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
                <div class="grid grid-cols-8 xl:grid-cols-10
                gap-1 xl:gap-x-3 xl:gap-y-2 px-2 xl:px-3 relative">
                    <For
                        each=move || 0..inventory.player_inventory.read().max_bag_size as usize
                        key=|i| *i
                        let(i)
                    >
                        <BagItem inventory=inventory.clone() item_index=i />
                    </For>
                </div>
            </CardInset>

        </MenuCard>
    }
}

#[component]
fn BagItem(inventory: InventoryConfig, item_index: usize) -> impl IntoView {
    let events_context: EventsContext = expect_context();
    let chat_context: ChatContext = expect_context();

    let is_being_equipped = RwSignal::new(false);

    let maybe_item = Signal::derive({
        let inventory = inventory.clone();
        move || {
            is_being_equipped.set(false);
            inventory
                .player_inventory
                .read()
                .bag
                .get(item_index)
                .cloned()
                .map(Arc::new)
        }
    });

    let can_equip = Signal::derive(move || {
        maybe_item
            .read()
            .as_ref()
            .map(|item_specs| {
                inventory
                    .equip_filter
                    .with(|equip_filter| match equip_filter {
                        InventoryEquipFilter::Slot => item_specs.base.slot.is_some(),
                        InventoryEquipFilter::Map(area_id) => item_specs
                            .base
                            .map_specs
                            .as_ref()
                            .map(|map_specs| {
                                map_specs
                                    .area_id
                                    .as_ref()
                                    .map(|map_area_id| *area_id == *map_area_id)
                                    .unwrap_or(true)
                            })
                            .unwrap_or_default(),
                        InventoryEquipFilter::Rune => item_specs.base.rune_specs.is_some(),
                    })
            })
            .unwrap_or_default()
    });

    let sell_queue = expect_context::<SellQueue>();
    let is_queued_for_sale = move || sell_queue.read().contains(&item_index);

    let show_menu = RwSignal::new(false);

    let item_ref = NodeRef::new();
    let tooltip_ref = NodeRef::new();

    let tooltip_size = Memo::new(move |_| {
        let tooltip_div: Option<web_sys::HtmlDivElement> = tooltip_ref.get();
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

        if tooltip_width > 0.0 {
            (
                (item_rect.left() - tooltip_width).max(0.0),
                item_rect.top().min(window_height - tooltip_height),
            )
        } else {
            (0.0, 0.0)
        }
    };

    view! {
        <div node_ref=item_ref class="relative group w-full aspect-[2/3]">
            {move || {
                match maybe_item.get() {
                    Some(item_specs) => {
                        let inventory = inventory.clone();
                        let comparable_item_specs = item_specs
                            .base
                            .slot
                            .and_then(|slot| {
                                inventory
                                    .player_inventory
                                    .read()
                                    .equipped
                                    .get(&slot)
                                    .and_then(|equipped_slot| match equipped_slot {
                                        EquippedSlot::MainSlot(item_specs) => {
                                            Some(Arc::from(item_specs.clone()))
                                        }
                                        EquippedSlot::ExtraSlot(_) => None,
                                    })
                            });

                        view! {
                            <div
                                class="relative w-full h-full overflow-visible"
                                class:brightness-50=move || !can_equip.get()
                            >
                                <ItemCard
                                    item_specs=item_specs.clone()
                                    comparable_item_specs=comparable_item_specs.clone()
                                    on:click={
                                        let chat_context = chat_context.clone();
                                        move |_| {
                                            if events_context.key_pressed(Key::Shift) {
                                                chat_context.link_item(item_specs.clone());
                                            } else {
                                                show_menu.set(true);
                                            }
                                        }
                                    }
                                    // Ignore if Mobile:
                                    on:contextmenu={
                                        let accessibility: AccessibilityContext = expect_context();
                                        move |ev| {
                                            ev.prevent_default();
                                            if !accessibility.is_on_mobile() {
                                                sell_queue
                                                    .update(|set| {
                                                        if !set.remove(&item_index) {
                                                            set.insert(item_index);
                                                        }
                                                    });
                                            }
                                        }
                                    }
                                    tooltip_position=DynamicTooltipPosition::AutoLeft
                                    max_item_level=inventory.max_item_level
                                />

                                <Show when=is_queued_for_sale>
                                    <div class="absolute top-1 right-1 z-20 px-1.5 xl:px-2 py-0.5 text-[10px] xl:text-xs font-black tracking-[0.08em] text-[#ffe0d3] border border-[#8e4538] rounded-[3px] shadow-[0_3px_8px_rgba(0,0,0,0.45),inset_0_1px_0_rgba(255,214,194,0.18)] bg-[linear-gradient(180deg,rgba(230,164,125,0.12),rgba(0,0,0,0.18)),linear-gradient(180deg,rgba(72,28,26,0.98),rgba(35,11,13,1))]">
                                        {match inventory.sell_type {
                                            SellType::Sell => "SELL",
                                            SellType::Discard => "DISC.",
                                        }}
                                    </div>
                                </Show>

                                <Show when=move || is_being_equipped.get()>
                                    <div
                                        class="absolute inset-0 z-30 w-full"
                                        style="clip-path: polygon(8px 0, calc(100% - 8px) 0, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 0 calc(100% - 8px), 0 8px);
                                        background:
                                        linear-gradient(180deg, rgba(214,177,102,0.04), rgba(0,0,0,0.08)),
                                        linear-gradient(135deg, rgba(32,31,36,0.82), rgba(8,8,10,0.92));
                                        box-shadow: inset 0 0 0 1px rgba(108,83,41,0.55), inset 0 0 18px rgba(0,0,0,0.45);"
                                    ></div>
                                </Show>

                                <Show when=move || { show_menu.get() }>
                                    <BagItemContextMenu
                                        inventory=inventory.clone()
                                        item_index=item_index
                                        on_close=Callback::new(move |_| show_menu.set(false))
                                        is_being_equipped=is_being_equipped
                                        can_equip
                                    />

                                    <Portal>
                                        <div
                                            node_ref=tooltip_ref
                                            class="fixed left-0 z-50 transition-opacity duration-150 text-center px-2"
                                            style=move || {
                                                let (x, y) = tooltip_pos();
                                                format!("left:{}px; top:{}px;", x, y)
                                            }
                                        >
                                            <ItemTooltip
                                                item_specs=maybe_item.get().unwrap().clone()
                                                max_item_level=inventory.max_item_level
                                            />
                                        </div>
                                    </Portal>

                                </Show>
                            </div>
                        }
                            .into_any()
                    }
                    None => view! { <EmptySlot>{}</EmptySlot> }.into_any(),
                }
            }}
        </div>
    }
}

#[component]
pub fn BagItemContextMenu(
    inventory: InventoryConfig,
    item_index: usize,
    on_close: Callback<()>,
    is_being_equipped: RwSignal<bool>,
    can_equip: Signal<bool>,
) -> impl IntoView {
    let sell_queue = expect_context::<SellQueue>();
    let (success_text, success_border, success_sheen) =
        action_menu_button_tone(ActionMenuTone::Success);
    let (warning_text, warning_border, warning_sheen) =
        action_menu_button_tone(ActionMenuTone::Warning);
    let (neutral_text, neutral_border, neutral_sheen) =
        action_menu_button_tone(ActionMenuTone::Neutral);

    let toggle_sell_mark = {
        move || {
            sell_queue.update(|set| {
                if !set.remove(&item_index) {
                    set.insert(item_index);
                }
            });
            on_close.run(());
        }
    };

    view! {
        <ContextMenu on_close=on_close>
            {{
                inventory
                    .on_equip
                    .and_then(|on_equip| {
                        can_equip
                            .get_untracked()
                            .then(|| {
                                view! {
                                    <button
                                        class=format!(
                                            "{} {}",
                                            action_menu_button_class(),
                                            success_text,
                                        )
                                        style=action_menu_button_style(
                                            success_border,
                                            success_sheen,
                                        )
                                        on:click=move |_| {
                                            on_equip(item_index as u8);
                                            sell_queue.write().remove(&item_index);
                                            is_being_equipped.set(true);
                                            set_timeout(
                                                move || is_being_equipped.set(false),
                                                Duration::from_millis(1000),
                                            );
                                            on_close.run(());
                                        }
                                    >
                                        "Equip"
                                    </button>
                                }
                            })
                    })
            }}
            {(inventory.on_sell.is_some())
                .then(|| {
                    view! {
                        <button
                            class=format!("{} {}", action_menu_button_class(), warning_text)
                            style=action_menu_button_style(warning_border, warning_sheen)
                            on:click=move |_| toggle_sell_mark()
                        >
                            {move || {
                                if sell_queue.get().contains(&item_index) {
                                    match inventory.sell_type {
                                        SellType::Sell => "Unsell",
                                        SellType::Discard => "Keep",
                                    }
                                } else {
                                    match inventory.sell_type {
                                        SellType::Sell => "Sell",
                                        SellType::Discard => "Discard",
                                    }
                                }
                            }}
                        </button>
                    }
                })}
            <button
                class=format!("{} {}", action_menu_button_class(), neutral_text)
                style=action_menu_button_style(neutral_border, neutral_sheen)
                on:click=move |_| on_close.run(())
            >
                "Cancel"
            </button>
        </ContextMenu>
    }
}

#[component]
fn EmptySlot(children: Children) -> impl IntoView {
    view! {
        <div class="relative isolate flex items-center justify-center w-full h-full overflow-hidden rounded-[4px] xl:rounded-[6px] opacity-80 border border-[#56462f]/80 shadow-[0_3px_7px_rgba(0,0,0,0.24),inset_0_1px_0_rgba(214,177,102,0.06),inset_0_-1px_0_rgba(0,0,0,0.38)] bg-[linear-gradient(180deg,rgba(214,177,102,0.03),rgba(0,0,0,0.12)),linear-gradient(135deg,rgba(39,38,44,0.94),rgba(15,15,18,1))]">
            <div class="pointer-events-none absolute inset-[1px] rounded-[3px] xl:rounded-[5px] border border-white/5 shadow-[inset_0_-10px_16px_rgba(0,0,0,0.2)]"></div>
            <div class="relative z-10 flex h-full w-full items-center justify-center p-1">
                {children()}
            </div>
        </div>
    }
}

#[component]
pub fn ContextMenu(on_close: Callback<()>, children: Children) -> impl IntoView {
    let node_ref = NodeRef::new();

    let _ = on_click_outside(node_ref, move |_| {
        on_close.run(());
    });

    view! {
        <div
            node_ref=node_ref
            class="
            absolute inset-0 z-30 flex flex-col justify-center items-center gap-1 xl:gap-2
            w-full
            p-1 xl:p-1.5
            text-center
            "
            style="
            animation: fade-in 0.2s ease-out forwards;
            clip-path: polygon(8px 0, calc(100% - 8px) 0, 100% 8px, 100% calc(100% - 8px), calc(100% - 8px) 100%, 8px 100%, 0 calc(100% - 8px), 0 8px);
            /*background:
            linear-gradient(180deg, rgba(214,177,102,0.08), rgba(0,0,0,0.16)),
            linear-gradient(135deg, rgba(42,40,46,0.96), rgba(17,16,20,0.98));
            border: 1px solid rgba(108,83,41,0.88);*/
            box-shadow:
            0 10px 22px rgba(0,0,0,0.52),
            inset 0 1px 0 rgba(240,215,159,0.16),
            inset 0 -1px 0 rgba(0,0,0,0.45);
            "
        >
            {children()}
        </div>
    }
}

#[derive(Clone, Copy)]
enum ActionMenuTone {
    Success,
    Warning,
    Neutral,
}

fn action_menu_button_tone(tone: ActionMenuTone) -> (&'static str, &'static str, &'static str) {
    match tone {
        ActionMenuTone::Success => (
            "text-green-300 hover:text-green-100",
            "#4c6a42",
            "rgba(178,228,173,0.12)",
        ),
        ActionMenuTone::Warning => (
            "text-amber-300 hover:text-amber-100",
            "#7b6032",
            "rgba(239,205,133,0.14)",
        ),
        ActionMenuTone::Neutral => (
            "text-stone-200 hover:text-white",
            "#665840",
            "rgba(240,215,159,0.08)",
        ),
    }
}

fn action_menu_button_class() -> &'static str {
    "btn relative isolate w-full overflow-hidden px-2 xl:px-2.5 py-1 xl:py-1.5
    text-sm xl:text-base font-extrabold tracking-[0.08em] text-shadow shadow-black/90
    border transition-all duration-150
    shadow-[0_4px_10px_rgba(0,0,0,0.42),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(240,215,159,0.12),inset_0_-1px_0_rgba(0,0,0,0.45)]
    before:pointer-events-none before:absolute before:inset-[1px] before:border before:border-white/5
    before:bg-[linear-gradient(180deg,rgba(255,255,255,0.04),transparent_36%)]
    hover:border-[#a27f46] active:translate-y-[1px] active:before:opacity-0 active:brightness-90
    active:shadow-[0_4px_10px_rgba(0,0,0,0.42),0_1px_0_rgba(26,17,10,0.95),inset_0_3px_5px_rgba(0,0,0,0.5),inset_0_-1px_0_rgba(0,0,0,0.22)]"
}

fn action_menu_button_style(border: &'static str, sheen: &'static str) -> String {
    format!(
        "border-color: {};
        background:
            linear-gradient(180deg, {}, rgba(0,0,0,0.16)),
            linear-gradient(180deg, rgba(43,40,46,0.96), rgba(20,19,23,1));",
        border, sheen,
    )
}

#[component]
fn SellAllButton(inventory: InventoryConfig) -> impl IntoView {
    inventory.on_sell.map(|on_sell| {
        let disabled = Signal::derive({
            let sell_queue = expect_context::<SellQueue>();
            move || sell_queue.read().is_empty()
        });
        view! {
            <MenuButton
                on:click={
                    let sell_queue = expect_context::<SellQueue>();
                    move |_| { on_sell(sell_queue.write().drain().map(|x| x as u8).collect()) }
                }
                disabled=disabled
            >
                <span class="inline xl:hidden">
                    {match inventory.sell_type {
                        SellType::Sell => "Sell all",
                        SellType::Discard => "Discard all",
                    }}
                </span>
                <span class="hidden xl:inline font-variant:small-caps">
                    {match inventory.sell_type {
                        SellType::Sell => "Sell all marked items",
                        SellType::Discard => "Discard all marked items",
                    }}
                </span>
            </MenuButton>
        }
    })
}

// #[component]
// pub fn LootFilterDropdown(loot_preference: RwSignal<Option<ItemCategory>>) -> impl IntoView {
//     let options = std::iter::once(None)
//         .chain(ItemCategory::iter().map(Some))
//         .map(|category| (category, loot_filter_category_to_str(category).into()))
//         .collect();

//     view! { <SearchableDropdownMenu options chosen_option=loot_preference /> }
// }

pub fn loot_filter_category_to_str(opt: Option<ItemCategory>) -> &'static str {
    use ItemCategory::*;
    match opt {
        Some(item_category) => match item_category {
            Armor => "Any Armor",
            Jewelry => "Any Jewelry",
            Accessory => "Any Accessory",
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
