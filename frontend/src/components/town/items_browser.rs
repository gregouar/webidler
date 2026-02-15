use chrono::{DateTime, Utc};
use leptos::{html::*, prelude::*};
use leptos_use::{UseInfiniteScrollOptions, use_infinite_scroll_with_options};
use std::sync::Arc;

use shared::data::{
    area::AreaLevel,
    item::{ItemSlot, ItemSpecs},
    player::EquippedSlot,
    user::UserCharacterId,
};

use crate::{
    assets::img_asset,
    components::{
        shared::{
            item_card::ItemCard,
            tooltips::{ItemTooltip, item_tooltip::ItemTooltipContent},
        },
        town::TownContext,
        ui::tooltip::{DynamicTooltipContext, DynamicTooltipPosition},
    },
};

#[derive(Clone)]
pub enum SelectedItem {
    None,
    InMarket(SelectedMarketItem),
    Removed(usize),
}

impl SelectedItem {
    pub fn is_empty(&self) -> bool {
        !matches!(self, SelectedItem::InMarket(_))
    }
}

#[derive(Clone)]
pub struct SelectedMarketItem {
    pub index: usize,
    pub item_specs: Arc<ItemSpecs>,
    pub price: f64,
    pub owner_id: Option<UserCharacterId>,
    pub owner_name: Option<String>,
    pub recipient: Option<(UserCharacterId, String)>,
    pub rejected: bool,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<(UserCharacterId, String)>,
}

#[component]
pub fn ItemsBrowser(
    selected_item: RwSignal<SelectedItem>,
    #[prop(into)] items_list: Signal<Vec<SelectedMarketItem>>,
    #[prop(optional)] reached_end_of_list: Option<RwSignal<bool>>,
    #[prop(optional)] has_more: Option<RwSignal<bool>>,
) -> impl IntoView {
    let town_context = expect_context::<TownContext>();
    let max_item_level = Signal::derive(move || town_context.character.read().max_area_level);

    let el = NodeRef::<Div>::new();
    if let Some(reached_end_of_list) = reached_end_of_list {
        use_infinite_scroll_with_options(
            el,
            move |_| async move {
                if !reached_end_of_list.get_untracked() {
                    reached_end_of_list.set(true)
                }
            },
            UseInfiniteScrollOptions::default().distance(20.0),
        );
    }
    view! {
        <div node_ref=el class="p-2 gap-2 overflow-y-auto h-full">
            <For
                each=move || items_list.get().into_iter()
                key=|item| (item.index,item.created_at)
                let:(item)
            >
                <ItemRow
                    item_specs=item.item_specs.clone()
                    on:click=move |_| selected_item.set(SelectedItem::InMarket(item.clone()))
                    price=item.price
                    highlight=move || selected_item.with(|selected_item| matches!(selected_item, SelectedItem::InMarket(selected_market_item) if selected_market_item.index == item.index))
                    special_offer=item.recipient.is_some()
                    rejected=item.rejected
                    max_item_level
                />
            </For>
            {move || (items_list.read().is_empty() && !has_more.map(|has_more| has_more.get()).unwrap_or_default()).then(|| view!{
                <div class="w-full h-full flex items-center justify-center">
                    <div class="flex flex-col items-center text-center gap-1">
                        <span class="text-gray-400">"No Item Found"</span>
                        <span class="text-gray-400">"Maybe try other filters?"</span>
                    </div>
                </div>
            })}
            {
                move || {
                    reached_end_of_list.and_then(|reached_end_of_list| {
                        (reached_end_of_list.get()
                            && has_more.map(|has_more| has_more.get()).unwrap_or_default())
                        .then(|| view! { <span class="text-gray-400">"Loading..."</span> })
                    })
                }
            }
        </div>
    }
}

#[component]
pub fn ItemRow(
    item_specs: Arc<ItemSpecs>,
    price: f64,
    highlight: impl Fn() -> bool + Send + Sync + 'static,
    #[prop(default = false)] special_offer: bool,
    #[prop(default = false)] rejected: bool,
    max_item_level: Signal<AreaLevel>,
) -> impl IntoView {
    let slot = item_specs.base.slot;
    view! {
        <div class=move || {
            format!(
                "relative flex w-full items-center justify-between p-3 gap-2 cursor-pointer mb-2 shadow-sm transition-colors duration-150 rounded-lg
                bg-neutral-800 hover:bg-neutral-700 {} {} {}",
                if highlight() { "ring-2 ring-amber-400" } else { "ring-1 ring-zinc-950" },
                if special_offer { "border-2 border-pink-400" } else { "" },
                if rejected { "border-2 border-red-500" } else { "" },
            )
        }>
            <div class="relative h-32 aspect-[2/3] flex-shrink-0">
                <ItemCard item_specs=item_specs.clone() class:pointer-events-none max_item_level />
            </div>

            <div class="flex flex-col w-full">
                <ItemTooltipContent
                    item_specs=item_specs.clone()
                    hide_description=true
                    max_item_level
                />
            </div>

            {slot.map(|slot| view! { <ItemCompare item_slot=slot max_item_level /> })}

            {(price > 0.0)
                .then(|| {
                    view! {
                        <div class="absolute flex bottom-2 right-2 gap-1 items-center">
                            <span class="text-gray-400">"Price:"</span>
                            <span class="text-fuchsia-300 font-semibold">
                                {format!("{:.0}", price)}
                            </span>
                            <img
                                draggable="false"
                                src=img_asset("ui/gems.webp")
                                alt="Gems"
                                class="h-[2em] aspect-square mr-1"
                            />
                        </div>
                    }
                })}
        </div>
    }
}

#[component]
pub fn ItemDetails(
    selected_item: RwSignal<SelectedItem>,
    #[prop(default = false)] show_affixes: bool,
) -> impl IntoView {
    let town_context: TownContext = expect_context();
    let max_item_level = Signal::derive(move || town_context.character.read().max_area_level);

    let item_details = move || {
        match selected_item.get() {
            SelectedItem::InMarket(selected_item) => {
                view! {
                    <div class="relative flex-shrink-0 w-1/4 aspect-[2/3]">
                        <ItemCard
                            item_specs=selected_item.item_specs.clone()
                            class:pointer-events-none
                            max_item_level
                        />
                    </div>

                    <div class="flex-1 w-full max-h-full overflow-y-auto">
                        <ItemTooltipContent
                            item_specs=selected_item.item_specs.clone()
                            class:select-text
                            show_affixes
                            max_item_level
                        />
                    </div>
                }
                .into_any()
            }
           SelectedItem:: None | SelectedItem::Removed(_) => {
                view! {
                    <div class="relative flex-shrink-0 w-1/4 aspect-[2/3]">
                        <div class="
                        relative group flex items-center justify-center w-full h-full
                        rounded-md border-2 border-zinc-700 bg-gradient-to-br from-zinc-800 to-zinc-900 opacity-70
                        "></div>
                    </div>

                    <div class="flex-1 text-gray-400">"No Item Selected"</div>
                }.into_any()
            }
        }
    };

    view! {
        <div class="w-full h-full flex items-center justify-center">
            <div class="flex flex-row gap-6 items-center
            w-full h-auto aspect-5/2 overflow-y-auto
            bg-neutral-800 rounded-lg  ring-1 ring-zinc-950  p-2">{item_details}</div>
        </div>
    }
}

#[component]
pub fn ItemCompare(item_slot: ItemSlot, max_item_level: Signal<AreaLevel>) -> impl IntoView {
    let tooltip_context: DynamicTooltipContext = expect_context();
    let town_context: TownContext = expect_context();

    let show_tooltip = move || {
        let item_specs = town_context
            .inventory
            .read()
            .equipped
            .get(&item_slot)
            .cloned();

        if let Some(EquippedSlot::MainSlot(item_specs)) = item_specs {
            let item_specs = Arc::new(*item_specs);
            tooltip_context.set_content(
                move || {
                    view! { <ItemTooltip item_specs=item_specs.clone() max_item_level /> }
                        .into_any()
                },
                DynamicTooltipPosition::Auto,
            );
        } else {
            tooltip_context.set_content(
                move || {
                    view! {
                        <div class="shadow-md bg-gradient-to-br from-gray-800 via-gray-900 to-black  p-2 xl:p-4 rounded-xl border">
                            "No item equipped"
                        </div>
                    }.into_any()
                },
                DynamicTooltipPosition::Auto,
            );
        }
    };

    let hide_tooltip = { move || tooltip_context.hide() };

    view! {
        <div
            class="absolute flex top-2 right-2 p-1 rounded-sm items-center bg-zinc-900 hover:bg-zinc-800"

            on:touchstart=move |_| { show_tooltip() }
            on:contextmenu=move |ev| {
                ev.prevent_default();
            }

            on:mouseenter=move |ev| {
                ev.prevent_default();
                show_tooltip()
            }
            on:mouseleave=move |_| hide_tooltip()
        >
            <span class="text-gray-400">"Compare"</span>
        </div>
    }
}
