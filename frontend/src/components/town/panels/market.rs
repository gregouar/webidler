use leptos::{html::*, prelude::*, task::spawn_local};

use std::{collections::HashSet, sync::Arc};

use shared::{
    data::{
        item::{ItemBase, ItemCategory, ItemRarity, ItemSpecs, WeaponSpecs},
        passive::{PassiveNodeId, PassiveNodeSpecs, PassivesTreeAscension},
    },
    http::client::AscendPassivesRequest,
};

use crate::{
    assets::img_asset,
    components::{
        auth::AuthContext,
        backend_client::BackendClient,
        game::{
            item_card::ItemCard,
            panels::passives::{Connection, MetaStatus, Node, NodeStatus, PurchaseStatus},
            tooltips::ItemTooltip,
        },
        town::TownContext,
        ui::{
            buttons::{CloseButton, MenuButton, TabButton},
            confirm::ConfirmContext,
            dropdown::DropdownMenu,
            menu_panel::MenuPanel,
            pannable::Pannable,
            toast::*,
            tooltip::DynamicTooltipPosition,
        },
    },
};

#[derive(Clone, Copy, PartialEq, Eq)]
enum MarketTab {
    Buy,
    Sell,
    Listings,
}

#[component]
pub fn MarketPanel(open: RwSignal<bool>) -> impl IntoView {
    let active_tab = RwSignal::new(MarketTab::Buy); // Buy or Sell
    let selected_item = RwSignal::new(None::<ItemSpecs>);

    view! {
        <MenuPanel open=open>
            <div class="w-full p-4">
                <div class="bg-zinc-800 rounded-md p-2 shadow-xl ring-1 ring-zinc-950 flex flex-col">
                    <div class="px-4 relative z-10 flex items-center justify-between">
                        <span class="text-shadow-md shadow-gray-950 text-amber-200 text-xl font-semibold mb-2">
                            "Market"
                        </span>

                        <div class="flex justify-center gap-4 w-full max-w-md mx-auto">
                            <TabButton
                                is_active=Signal::derive(move || active_tab.get() == MarketTab::Buy)
                                on:click=move |_| { active_tab.set(MarketTab::Buy) }
                            >
                                "Buy"
                            </TabButton>
                            <TabButton
                                is_active=Signal::derive(move || {
                                    active_tab.get() == MarketTab::Sell
                                })
                                on:click=move |_| { active_tab.set(MarketTab::Sell) }
                            >
                                "Sell"
                            </TabButton>
                            <TabButton
                                is_active=Signal::derive(move || {
                                    active_tab.get() == MarketTab::Listings
                                })
                                on:click=move |_| { active_tab.set(MarketTab::Listings) }
                            >
                                "My Listings"
                            </TabButton>
                        </div>

                        <div class="flex items-center gap-2 mb-2">
                            <CloseButton on:click=move |_| open.set(false) />
                        </div>
                    </div>

                    <div class="w-full aspect-[4/3] sm:aspect-[3/2] md:aspect-[5/2] overflow-hidden bg-neutral-900">
                        <Market />
                    </div>

                    <div class="px-4 relative z-10 flex items-center justify-between"></div>
                </div>
            </div>
        </MenuPanel>
    }
}

#[component]
pub fn Market() -> impl IntoView {
    view! {}
}

// #[component]
// pub fn MarketPanel(open: RwSignal<bool>) -> impl IntoView {
//     let active_tab = RwSignal::new(MarketTab::Buy); // Buy or Sell
//     let selected_item = RwSignal::new(None::<ItemSpecs>);

//     let current_page = RwSignal::new(1usize);
//     let total_pages = RwSignal::new(10usize); // TODO: fetch from server

//     view! {
//         <MenuPanel open=open>
//             <div class="flex flex-col w-full h-full p-4 gap-4 relative">
//                 // Close button like InventoryPanel
//                 <div class="absolute top-2 right-2">
//                     <CloseButton on:click=move |_| open.set(false) />
//                 </div>

//                 // Tabs
//                 <div class="flex justify-center gap-4">
//                     <button
//                         class=move || tab_class(active_tab.get() == MarketTab::Buy)
//                         on:click=move |_| {
//                             active_tab.set(MarketTab::Buy);
//                             current_page.set(1);
//                         }
//                     >
//                         "Buy"
//                     </button>
//                     <button
//                         class=move || tab_class(active_tab.get() == MarketTab::Sell)
//                         on:click=move |_| {
//                             active_tab.set(MarketTab::Sell);
//                             current_page.set(1);
//                         }
//                     >
//                         "Sell"
//                     </button>
//                 </div>

//                 // Panel body
//                 <div class="grid grid-cols-7 gap-4 h-[75vh]">
//                     <MarketBrowser
//                         active_tab
//                         selected_item
//                         current_page
//                         total_pages
//                         class:col-span-4
//                     />
//                     <MarketDetails active_tab selected_item class:col-span-3 />
//                 </div>

//                 // Pagination footer
//                 <PaginationControls current_page total_pages />
//             </div>
//         </MenuPanel>
//     }
// }

// // #[component]
// // fn PaginationControls(
// //     current_page: RwSignal<usize>,
// //     total_pages: RwSignal<usize>,
// // ) -> impl IntoView {
// //     const MAX_VISIBLE: usize = 5; // max visible numbered buttons

// //     view! {
// //         <div class="flex justify-center items-center gap-2 mt-2 select-none">
// //             // Previous Arrow
// //             <button
// //                 class="px-2 py-1 rounded bg-zinc-700 text-white disabled:opacity-40"
// //                 disabled=move || current_page.get() == 1
// //                 on:click=move |_| {
// //                     current_page
// //                         .update(|p| {
// //                             if *p > 1 {
// //                                 *p -= 1;
// //                             }
// //                         })
// //                 }
// //             >
// //                 "‹"
// //             </button>

// //             // Numbered Pages
// //             {move || {
// //                 let page = current_page.get();
// //                 let total = total_pages.get();
// //                 let mut buttons: Vec<_> = vec![];
// //                 let start_page = if total <= MAX_VISIBLE {
// //                     1
// //                 } else if page <= 3 {
// //                     1
// //                 } else if page >= total - 2 {
// //                     total - MAX_VISIBLE + 1
// //                 } else {
// //                     page - 2
// //                 };
// //                 let end_page = if total <= MAX_VISIBLE {
// //                     total
// //                 } else if page <= 3 {
// //                     MAX_VISIBLE
// //                 } else if page >= total - 2 {
// //                     total
// //                 } else {
// //                     page + 2
// //                 };
// //                 if start_page > 1 {
// //                     buttons.push(page_button(1, page, current_page.clone()));
// //                     if start_page > 2 {
// //                         buttons.push(ellipsis());
// //                     }
// //                 }
// //                 for p in start_page..=end_page {
// //                     buttons.push(page_button(p, page, current_page.clone()));
// //                 }
// //                 if end_page < total {
// //                     if end_page < total - 1 {
// //                         buttons.push(ellipsis());
// //                     }
// //                     buttons.push(page_button(total, page, current_page.clone()));
// //                 }

// //                 // Show first page + ellipsis if needed

// //                 // Main range

// //                 // Show ellipsis + last page if needed

// //                 view! { {buttons} }
// //             }}

// //             // Next Arrow
// //             <button
// //                 class="px-2 py-1 rounded bg-zinc-700 text-white disabled:opacity-40"
// //                 disabled=move || current_page.get()
// //             >
// //                 = total_pages.get()
// //                 on:click=move |_| current_page.update(|p| if *p < total_pages.get() { *p += 1 })
// //                 >
// //                 "›"
// //             </button>
// //         </div>
// //     }
// // }

// // fn page_button(p: usize, current: usize, current_page: RwSignal<usize>) -> impl IntoView {
// //     let active = p == current;
// //     let class = if active {
// //         "px-3 py-1 rounded bg-blue-500 text-white font-bold"
// //     } else {
// //         "px-3 py-1 rounded bg-zinc-700 text-gray-200 hover:bg-zinc-600"
// //     };

// //     view! {
// //         <button class=class on:click=move |_| current_page.set(p)>
// //             {p}
// //         </button>
// //     }
// // }

// // fn ellipsis() -> impl IntoView {
// //     view! { <span class="px-2 text-gray-400">"..."</span> }
// // }

// #[component]
// fn PaginationControls(
//     current_page: RwSignal<usize>,
//     total_pages: RwSignal<usize>,
// ) -> impl IntoView {
//     view! {
//         <div class="flex justify-center items-center gap-2 mt-2">
//             <button
//                 class="px-3 py-1 bg-zinc-700 rounded text-white disabled:opacity-40"
//                 disabled=move || current_page.get() == 1
//                 on:click=move |_| {
//                     current_page
//                         .update(|p| {
//                             if *p > 1 {
//                                 *p -= 1;
//                             }
//                         })
//                 }
//             >
//                 "Previous"
//             </button>

//             <span class="text-gray-300">
//                 {move || format!("Page {} of {}", current_page.get(), total_pages.get())}
//             </span>

//             <button
//                 class="px-3 py-1 bg-zinc-700 rounded text-white disabled:opacity-40"
//                 disabled=move || current_page.get()
//             >
//                 = total_pages.get()
//                 on:click=move |_| current_page.update(|p| *p += 1)
//                 >
//                 "Next"
//             </button>
//         </div>
//     }
// }

// fn tab_class(active: bool) -> &'static str {
//     if active {
//         "px-4 py-2 text-white bg-zinc-700 rounded-t-md font-bold"
//     } else {
//         "px-4 py-2 text-gray-400 hover:text-white hover:bg-zinc-800 rounded-t-md"
//     }
// }

// #[component]
// fn MarketBrowser(
//     active_tab: RwSignal<MarketTab>,
//     selected_item: RwSignal<Option<ItemSpecs>>,
//     current_page: RwSignal<usize>,
//     total_pages: RwSignal<usize>,
// ) -> impl IntoView {
//     let search_term = RwSignal::new(String::new());
//     let category_filter = RwSignal::new(None::<ItemCategory>);
//     let rarity_filter = RwSignal::new(None::<String>); // Example

//     let mock_market_items = vec![ItemSpecs {
//         name: "Sword".into(),
//         base: ItemBase {
//             name: "Sword".into(),
//             icon: "items/wool_cloak.webp".into(),
//             description: None,
//             slot: shared::data::item::ItemSlot::Accessory,
//             extra_slots: HashSet::new(),
//             categories: HashSet::new(),
//             min_area_level: 1,
//             rarity: ItemRarity::Normal,
//             affixes: Vec::new(),
//             triggers: Vec::new(),
//             weapon_specs: None,
//             armor_specs: None,
//         },
//         rarity: ItemRarity::Normal,
//         level: 2,
//         weapon_specs: None,
//         armor_specs: None,
//         affixes: Vec::new(),
//         triggers: Vec::new(),
//         old_game: true,
//     }];

//     view! {
//         <div class="bg-zinc-800 rounded-md p-2 shadow-lg ring-1 ring-zinc-950 flex flex-col h-full">
//             <div class="flex-1 overflow-y-auto bg-neutral-900 p-2 rounded shadow-inner">
//                 <For
//                     // Replace with actual source
//                     each=move || mock_market_items.clone().into_iter().enumerate()
//                     key=|(k,_)| *k
//                     let:((_,item))
//                 >
//                     <div
//                         class="flex items-center gap-3 p-2 bg-zinc-700 hover:bg-zinc-600 rounded cursor-pointer mb-1"
//                         on:click=move |_| selected_item.set(Some(item.clone()))
//                     >
//                         <img
//                             src=img_asset(&item.base.icon)
//                             alt="icon"
//                             class="w-10 h-10 object-contain"
//                         />
//                         <div class="flex flex-col text-sm">
//                             <span class="font-semibold text-white">{item.base.name.clone()}</span>
//                             <span class="text-gray-400">
//                                 {format!(
//                                     "Armor: {:.0}",
//                                     item.armor_specs.as_ref().map(|a| a.armor).unwrap_or(0.0),
//                                 )} {" | "}
//                                 {format!(
//                                     "Damage: {}",
//                                     item
//                                         .weapon_specs
//                                         .as_ref()
//                                         .map(|w| total_damage_range(w))
//                                         .unwrap_or("-".into()),
//                                 )} {" | "} {format!("Lvl {}", item.level)}
//                             </span>
//                         </div>
//                     </div>
//                 </For>
//             </div>
//         </div>
//     }
// }

// fn total_damage_range(w: &WeaponSpecs) -> String {
//     let (mut min, mut max) = (0.0, 0.0);
//     for (_, (lo, hi)) in &w.damage {
//         min += lo;
//         max += hi;
//     }
//     format!("{:.0}-{:.0}", min, max)
// }

// #[component]
// fn MarketDetails(
//     active_tab: RwSignal<MarketTab>,
//     selected_item: RwSignal<Option<ItemSpecs>>,
// ) -> impl IntoView {
//     view! {
//         <div class="bg-zinc-800 rounded-md p-4 shadow-lg ring-1 ring-zinc-950 flex flex-col items-center">
//             <Show
//                 when=move || selected_item.get().is_some()
//                 fallback=|| {
//                     view! { <div class="text-gray-400 mt-4">"Select an item to see details"</div> }
//                 }
//             >
//                 {move || {
//                     let item = selected_item.get().unwrap();
//                     view! {
//                         <img src=img_asset(&item.base.icon) class="w-24 h-24 mb-4 object-contain" />
//                         <ItemTooltip item_specs=Arc::new(item.clone()) />
//                         <div class="mt-4">
//                             {if active_tab.get() == MarketTab::Buy {
//                                 view! {
//                                     <button class="px-4 py-2 bg-green-600 text-white rounded">
//                                         "Buy"
//                                     </button>
//                                 }
//                             } else {
//                                 view! {
//                                     <button class="px-4 py-2 bg-red-600 text-white rounded">
//                                         "Sell"
//                                     </button>
//                                 }
//                             }}
//                         </div>
//                     }
//                 }}
//             </Show>
//         </div>
//     }
// }

// ------------------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------------------
// ------------------------------------------------------------------------------------------------------------

// #[component]
// pub fn MarketPanel(open: RwSignal<bool>) -> impl IntoView {
//     let active_tab = RwSignal::new("Buy".to_string());
//     let selected_item = RwSignal::new(None::<ItemSpecs>);

//     let mock_items = vec![ItemSpecs {
//         name: "Sword".into(),
//         base: ItemBase {
//             name: "Sword".into(),
//             icon: "items/wool_cloak.webp".into(),
//             description: None,
//             slot: shared::data::item::ItemSlot::Accessory,
//             extra_slots: HashSet::new(),
//             categories: HashSet::new(),
//             min_area_level: 1,
//             rarity: ItemRarity::Normal,
//             affixes: Vec::new(),
//             triggers: Vec::new(),
//             weapon_specs: None,
//             armor_specs: None,
//         },
//         rarity: ItemRarity::Normal,
//         level: 2,
//         weapon_specs: None,
//         armor_specs: None,
//         affixes: Vec::new(),
//         triggers: Vec::new(),
//         old_game: true,
//     }];

//     view! {
//         <MenuPanel open=open>
//             <div class="bg-zinc-800 rounded-md p-2 shadow-lg ring-1 ring-zinc-950 flex flex-col gap-2 h-full w-full">
//                 // Header
//                 <div class="px-4 flex items-center justify-between gap-2">
//                     <span class="text-amber-200 text-xl font-semibold">"Market"</span>
//                     <CloseButton on:click=move |_| open.set(false) />
//                 </div>

//                 // Tabs
//                 <div class="flex gap-2 px-4">
//                     <MenuButton
//                         text="Buy"
//                         active=move || active_tab.get() == "Buy"
//                         on_click=move |_| active_tab.set("Buy".to_string())
//                     />
//                     <MenuButton
//                         text="Sell"
//                         active=move || active_tab.get() == "Sell"
//                         on_click=move |_| active_tab.set("Sell".to_string())
//                     />
//                 </div>

//                 // Content area
//                 <div class="flex-1 grid grid-cols-2 gap-4 p-4">
//                     // Left: Browsing Area
//                     <div class="flex flex-col gap-2 p-2 bg-neutral-900 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)] rounded-md overflow-y-auto">
//                         <ItemFilterBar />
//                         <For each=mock_items.clone() key=|item| item.name.clone() let:item>
//                             <MarketItemRow item=item.clone() price=120.0 />
//                         </For>
//                     </div>

//                     // Right: Item Detail and Actions
//                     <MarketItemDetailPanel item=selected_item />
//                 </div>
//             </div>
//         </MenuPanel>
//     }
// }

// // #[component]
// // pub fn MarketPanel(open: RwSignal<bool>) -> impl IntoView {
// //     let active_tab = RwSignal::new("Buy".to_string()); // or "Sell"
// //     let selected_item = RwSignal::new(None::<ItemSpecs>);

// //     view! {
// //         <MenuPanel open=open>
// //             <div class="bg-zinc-800 rounded-md p-2 shadow-lg ring-1 ring-zinc-950 flex flex-col gap-2 h-full w-full">

// //                 // Header
// //                 <div class="px-4 flex items-center justify-between gap-2">
// //                     <span class="text-amber-200 text-xl font-semibold">"Market"</span>
// //                     <CloseButton on:click=move |_| open.set(false) />
// //                 </div>

// //                 // Tabs
// //                 <div class="flex gap-2 px-4">
// //                     <MenuButton
// //                         text="Buy"
// //                         active=move || active_tab.get() == "Buy"
// //                         on_click=move |_| active_tab.set("Buy".to_string())
// //                     />
// //                     <MenuButton
// //                         text="Sell"
// //                         active=move || active_tab.get() == "Sell"
// //                         on_click=move |_| active_tab.set("Sell".to_string())
// //                     />
// //                 </div>

// //                 // Main content
// //                 <div class="flex-1 grid grid-cols-2 gap-4 p-4">
// //                     // Browsing area (list of items)
// //                     <div class="flex flex-col gap-2 p-2 bg-neutral-900 shadow-[inset_0_0_32px_rgba(0,0,0,0.6)] rounded-md overflow-y-auto">
// //                         <ItemFilterBar />
// //                         <For each=mock_items key=|item| item.name.clone() let:item>
// //                             <MarketItemRow item=item.clone() price=120.0 />
// //                         </For>
// //                     </div>

// //                     // Detail + action area
// //                     <Show when=move || selected_item.get().is_some()>
// //                         <MarketItemDetail item=selected_item.get().unwrap() />
// //                     </Show>
// //                 </div>
// //             </div>
// //         </MenuPanel>
// //     }
// // }

// #[component]
// pub fn MarketItemDetailPanel(item: RwSignal<Option<ItemSpecs>>) -> impl IntoView {
//     view! {
//         <div class="bg-zinc-700 rounded-md p-4 shadow-inner flex flex-col gap-4">
//             <Show
//                 when=move || item.get().is_some()
//                 fallback=|| {
//                     view! { <span class="text-gray-400">"Select an item to view details."</span> }
//                 }
//             >
//                 {move || {
//                     let itm = item.get().unwrap();
//                     view! {
//                         <div class="flex items-center gap-4">
//                             <img
//                                 src=img_asset(&itm.base.icon)
//                                 class="w-16 h-16 rounded-md shadow-md"
//                             />
//                             <div>
//                                 <span class="text-lg font-bold">{itm.name.clone()}</span>
//                                 <p class="text-sm text-gray-400 mt-1">
//                                     {itm.base.description.clone().unwrap_or_default()}
//                                 </p>
//                             </div>
//                         </div>
//                         <div class="flex gap-2">
//                             <MenuButton text="Buy" on_click=move |_| {} />
//                             <MenuButton text="Sell" on_click=move |_| {} />
//                         </div>
//                     }
//                 }}
//             </Show>
//         </div>
//     }
// }

// #[component]
// fn TabButton(label: &'static str, active_tab: RwSignal<usize>, index: usize) -> impl IntoView {
//     let is_active = move || active_tab.get() == index;
//     view! {
//         <button
//             class=move || {
//                 format!(
//                     "px-4 py-2 rounded-md font-bold {}",
//                     if is_active() { "bg-amber-600" } else { "bg-zinc-700 hover:bg-zinc-600" },
//                 )
//             }
//             on:click=move |_| active_tab.set(index)
//         >
//             {label}
//         </button>
//     }
// }

// #[component]
// fn MarketTabBuy() -> impl IntoView {
//     view! {
//         <div class="grid grid-cols-7 gap-4 h-full">
//             // Left: Browsing
//             <MarketBrowsingPanel class:col-span-4 mode="buy" />
//             // Right: Details & Actions
//             <MarketDetailsPanel class:col-span-3 mode="buy" />
//         </div>
//     }
// }

// #[component]
// fn MarketTabSell() -> impl IntoView {
//     view! {
//         <div class="grid grid-cols-7 gap-4 h-full">
//             <MarketBrowsingPanel class:col-span-4 mode="sell" />
//             <MarketDetailsPanel class:col-span-3 mode="sell" />
//         </div>
//     }
// }
// #[component]
// fn MarketBrowsingPanel(mode: &'static str) -> impl IntoView {
//     view! {
//         <div class="flex flex-col bg-zinc-800 p-4 rounded-md h-full shadow-xl ring-1 ring-zinc-950">
//             <div class="flex gap-2 mb-3">
//                 <input class="bg-zinc-900 rounded px-2 py-1" placeholder="Search name..." />
//                 <select class="bg-zinc-900 rounded px-2 py-1">
//                     <option>"Category"</option>
//                     <option>"Weapons"</option>
//                     <option>"Armor"</option>
//                     <option>"Consumables"</option>
//                 </select>
//                 <select class="bg-zinc-900 rounded px-2 py-1">
//                     <option>"Rarity"</option>
//                     <option>"Common"</option>
//                     <option>"Rare"</option>
//                     <option>"Epic"</option>
//                 </select>
//             </div>
//             // Item list scrollable
//             // <MarketItemRow name="Iron Sword" price=120.0 />
//             // <MarketItemRow name="Steel Armor" price=250.0 />
//             // ...
//             <div class="overflow-y-auto flex-1 space-y-2"></div>
//         </div>
//     }
// }

// #[component]
// pub fn ItemFilterBar() -> impl IntoView {
//     view! {
//         <div class="flex gap-2 mb-2 flex-wrap">
//             <input
//                 type="text"
//                 placeholder="Search..."
//                 class="bg-zinc-700 text-gray-200 text-sm px-2 py-1 rounded-md outline-none"
//             />
//             <select class="bg-zinc-700 text-gray-200 text-sm px-2 py-1 rounded-md">
//                 <option value="">"All Categories"</option>
//                 <option value="Armor">"Armor"</option>
//                 <option value="Weapon">"Weapon"</option>
//             </select>
//             <select class="bg-zinc-700 text-gray-200 text-sm px-2 py-1 rounded-md">
//                 <option value="">"All Rarities"</option>
//                 <option value="Normal">"Normal"</option>
//                 <option value="Magic">"Magic"</option>
//                 <option value="Rare">"Rare"</option>
//                 <option value="Unique">"Unique"</option>
//             </select>
//         </div>
//     }
// }

// #[component]
// fn MarketDetailsPanel(mode: &'static str) -> impl IntoView {
//     view! {
//         <div class="bg-neutral-900 rounded-md p-4 h-full flex flex-col justify-between shadow-xl ring-1 ring-zinc-950">
//             <div>
//                 <h3 class="text-xl font-bold mb-2">"Item Details"</h3>
//                 <div class="text-sm text-gray-400">"Select an item to view stats"</div>
//             </div>
//             <div class="mt-4">
//                 <button class="w-full bg-emerald-600 hover:bg-emerald-500 text-white py-2 rounded-md">
//                     {if mode == "buy" { "Buy Item" } else { "List for Sale" }}
//                 </button>
//             </div>
//         </div>
//     }
// }

// #[component]
// pub fn MarketItemRow(item: ItemSpecs, price: f64) -> impl IntoView {
//     let rarity_color = match item.rarity {
//         ItemRarity::Normal => "text-gray-300",
//         ItemRarity::Magic => "text-blue-400",
//         ItemRarity::Rare => "text-yellow-400",
//         ItemRarity::Unique => "text-orange-400",
//     };

//     view! {
//         <div
//             class="flex items-center justify-between bg-zinc-700 hover:bg-zinc-600 rounded-md px-3 py-2 cursor-pointer transition-colors"
//             on:click=move |_| {}
//         >
//             <div class="flex items-center gap-3">
//                 <img src=img_asset(&item.base.icon) class="w-10 h-10 rounded-md shadow-md" />
//                 <div class="flex flex-col">
//                     <span class=format!("font-bold {}", rarity_color)>{item.name.clone()}</span>
//                     <span class="text-xs text-gray-400">{format!("{:?}", item.base.slot)}</span>
//                 </div>
//             </div>
//             <span class="text-emerald-400 font-semibold">{format!("{:.0} Gold", price)}</span>
//         </div>
//     }
// }

// // #[component]
// // pub fn MarketItemRow(item: ItemSpecs, price: f64) -> impl IntoView {
// //     let rarity_color = match item.rarity {
// //         ItemRarity::Normal => "text-gray-300",
// //         ItemRarity::Magic => "text-blue-400",
// //         ItemRarity::Rare => "text-yellow-400",
// //         ItemRarity::Unique => "text-orange-400",
// //     };

// //     view! {
// //         <div
// //             class="flex items-center justify-between bg-zinc-700 hover:bg-zinc-600 rounded-md px-3 py-2 cursor-pointer transition-colors"
// //             on:click=move |_| {}
// //         >
// //             <div class="flex items-center gap-3">
// //                 <img src=img_asset(&item.base.icon) class="w-10 h-10 rounded-md shadow-md" />
// //                 <div class="flex flex-col">
// //                     <span class=format!("font-bold {}", rarity_color)>{item.name.clone()}</span>
// //                     <span class="text-xs text-gray-400">{format!("{:?}", item.base.slot)}</span>
// //                 </div>
// //             </div>
// //             <span class="text-emerald-400 font-semibold">{format!("{:.0} Gold", price)}</span>
// //         </div>
// //     }
// // }

// // #[component]
// // pub fn MarketItemRow(item: ItemSpecs, price: f64) -> impl IntoView {
// //     // Map rarity to color
// //     let rarity_color = match item.rarity {
// //         ItemRarity::Normal => "text-gray-300",
// //         ItemRarity::Magic => "text-blue-400",
// //         ItemRarity::Rare => "text-yellow-400",
// //         ItemRarity::Unique => "text-orange-400",
// //     };

// //     view! {
// //         <div
// //             class="flex items-center justify-between bg-zinc-900 hover:bg-zinc-800 rounded-md px-3 py-2 cursor-pointer transition-colors"
// //             on:click=move |_| {}
// //         >
// //             // Item Icon + Name
// //             <div class="flex items-center gap-3">
// //                 <img src=img_asset(&item.base.icon) class="w-10 h-10 rounded-md shadow-md" />
// //                 <span class=format!("font-bold {}", rarity_color)>{item.name.clone()}</span>
// //             </div>

// //             // Item Price
// //             <div class="text-emerald-400 font-semibold">{format!("{:.0} Gold", price)}</div>
// //         </div>
// //     }
// // }
