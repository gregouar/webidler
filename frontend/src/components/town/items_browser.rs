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
        ui::{
            list_row::MenuListRow,
            tooltip::{DynamicTooltipContext, DynamicTooltipPosition},
        },
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
pub fn BrowserEmptyItemSlot() -> impl IntoView {
    view! {
        <div class="relative isolate flex items-center justify-center w-full h-full overflow-hidden rounded-[6px]
        border border-[#56462f]/80
        bg-[linear-gradient(180deg,rgba(214,177,102,0.03),rgba(0,0,0,0.12)),linear-gradient(135deg,rgba(39,38,44,0.94),rgba(15,15,18,1))]
        shadow-[0_3px_7px_rgba(0,0,0,0.24),inset_0_1px_0_rgba(214,177,102,0.06),inset_0_-1px_0_rgba(0,0,0,0.38)]">
            <div class="pointer-events-none absolute inset-[1px] rounded-[5px] border border-white/5 shadow-[inset_0_-10px_16px_rgba(0,0,0,0.2)]"></div>
            <div class="relative z-10 flex h-full w-full items-center justify-center p-1 text-center"></div>
        </div>
    }
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
                {{
                    let clicked_item = item.clone();
                    view! {
                        <ItemRow
                            item_specs=item.item_specs.clone()
                            on_click=move || selected_item.set(SelectedItem::InMarket(clicked_item.clone()))
                            price=item.price
                            highlight=move || selected_item.with(|selected_item| matches!(selected_item, SelectedItem::InMarket(selected_market_item) if selected_market_item.index == item.index))
                            special_offer=item.recipient.is_some()
                            rejected=item.rejected
                            max_item_level
                        />
                    }
                }}
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
    #[prop(optional, into)] on_click: Option<Callback<()>>,
    highlight: impl Fn() -> bool + Send + Sync + 'static,
    #[prop(default = false)] special_offer: bool,
    #[prop(default = false)] rejected: bool,
    max_item_level: Signal<AreaLevel>,
) -> impl IntoView {
    let slot = item_specs.base.slot;
    let is_highlighted = Signal::derive(highlight);
    let row_state_class = Signal::derive(move || {
        let mut classes = String::new();

        if is_highlighted.get() {
            classes.push_str(
                "border-[#b28a4f] shadow-[0_4px_14px_rgba(0,0,0,0.28),inset_0_1px_0_rgba(255,255,255,0.04),inset_0_0_0_1px_rgba(214,177,102,0.18)] ",
            );
        } else if rejected {
            classes.push_str(
                "border-red-700/70 shadow-[0_4px_12px_rgba(0,0,0,0.24),inset_0_1px_0_rgba(255,255,255,0.03),inset_0_0_0_1px_rgba(239,68,68,0.12)] ",
            );
        } else if special_offer {
            classes.push_str(
                "border-fuchsia-700/70 shadow-[0_4px_12px_rgba(0,0,0,0.24),inset_0_1px_0_rgba(255,255,255,0.03),inset_0_0_0_1px_rgba(217,70,239,0.14)] ",
            );
        }

        if special_offer {
            classes.push_str(
                "before:absolute before:inset-y-[8px] before:left-0 before:w-[2px] before:rounded-full before:bg-gradient-to-b before:from-transparent before:via-fuchsia-400/80 before:to-transparent ",
            );
        }

        if rejected {
            classes.push_str(
                "after:absolute after:inset-y-[8px] after:left-[5px] after:w-[2px] after:rounded-full after:bg-gradient-to-b after:from-transparent after:via-red-400/85 after:to-transparent",
            );
        }

        classes
    });

    view! {
        <MenuListRow
            class="mb-2"
            state_class=row_state_class
            selected=is_highlighted
            on_click=move || {
                if let Some(on_click) = on_click {
                    on_click.run(());
                }
            }
        >
            <div class="flex w-full items-center justify-between gap-3 px-3 py-3">
                <div class="relative h-28 xl:h-32 aspect-[2/3] flex-shrink-0">
                    <ItemCard
                        item_specs=item_specs.clone()
                        class:pointer-events-none
                        max_item_level
                    />
                </div>

                <div class="min-w-0 flex flex-col w-full">
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
                                <span class="text-gray-400 text-xs xl:text-sm">"Price:"</span>
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
        </MenuListRow>
    }
}

#[component]
pub fn ItemDetails(
    selected_item: RwSignal<SelectedItem>,
    #[prop(default = false)] show_affixes: bool,
) -> impl IntoView {
    let town_context: TownContext = expect_context();
    let max_item_level = Signal::derive(move || town_context.character.read().max_area_level);

    let selected_specs = Signal::derive(move || match selected_item.get() {
        SelectedItem::InMarket(selected_item) => Some(selected_item.item_specs.clone()),
        SelectedItem::None | SelectedItem::Removed(_) => None,
    });

    view! {
        <ItemDetailsPanel
            item_specs=selected_specs
            show_affixes
            max_item_level
            empty_label="No Item Selected"
        />
    }
}

#[component]
pub fn ItemDetailsPanel(
    #[prop(into)] item_specs: Signal<Option<Arc<ItemSpecs>>>,
    #[prop(default = false)] show_affixes: bool,
    max_item_level: Signal<AreaLevel>,
    empty_label: &'static str,
    #[prop(optional)] empty_label_class: Option<&'static str>,
    #[prop(optional, into)] selected: Option<Signal<bool>>,
    #[prop(optional, into)] on_click: Option<Callback<()>>,
) -> impl IntoView {
    let is_selected =
        Signal::derive(move || selected.map(|selected| selected.get()).unwrap_or(false));
    let is_clickable = on_click.is_some();

    view! {
        <div class="w-full h-full flex items-center justify-center">
            <div
                class=move || {
                    format!(
                        "relative isolate w-full h-auto aspect-5/2 overflow-hidden rounded-[10px] border
                        bg-[linear-gradient(180deg,rgba(226,193,122,0.05),rgba(0,0,0,0.02)_28%,rgba(0,0,0,0.14)_100%),linear-gradient(135deg,rgba(40,39,45,0.98),rgba(18,18,22,1))]
                        shadow-[0_6px_16px_rgba(0,0,0,0.22),inset_0_1px_0_rgba(255,255,255,0.04),inset_0_-1px_0_rgba(0,0,0,0.35)]
                        transition-[border-color,background-color,box-shadow,transform] duration-150
                        {} {}",
                        if is_selected.get() {
                            "border-[#b28a4f] shadow-[0_6px_18px_rgba(0,0,0,0.26),inset_0_1px_0_rgba(244,225,181,0.07),inset_0_0_0_1px_rgba(214,177,102,0.16)]"
                        } else {
                            "border-[#3b3428]"
                        },
                        if is_clickable {
                            "cursor-pointer hover:border-[#75603c] hover:bg-[linear-gradient(180deg,rgba(226,193,122,0.065),rgba(0,0,0,0.02)_28%,rgba(0,0,0,0.14)_100%),linear-gradient(135deg,rgba(46,45,52,0.99),rgba(22,22,27,1))]"
                        } else {
                            ""
                        },
                    )
                }
                on:click=move |_| {
                    if let Some(on_click) = on_click {
                        on_click.run(());
                    }
                }
            >
                <div class="pointer-events-none absolute inset-[1px] rounded-[9px] border border-white/5"></div>
                <div class="pointer-events-none absolute inset-x-4 top-0 h-px bg-gradient-to-r from-transparent via-[#edd39a]/40 to-transparent"></div>

                <div class="relative z-10 flex h-full w-full min-h-0 flex-row gap-4 xl:gap-6 p-3 xl:p-4">
                    <div class="flex w-1/4 max-w-[12rem] flex-shrink-0 items-center">
                        <div class="relative w-full aspect-[2/3]">
                            {move || {
                                item_specs
                                    .get()
                                    .map(|item_specs| {
                                        view! {
                                            <ItemCard
                                                item_specs=item_specs
                                                class:pointer-events-none
                                                max_item_level
                                            />
                                        }
                                            .into_any()
                                    })
                                    .unwrap_or_else(|| {
                                        view! { <BrowserEmptyItemSlot /> }.into_any()
                                    })
                            }}
                        </div>
                    </div>

                    <div class="flex-1 min-w-0 self-stretch overflow-y-auto pr-1">
                        {move || {
                            item_specs
                                .get()
                                .map(|item_specs| {
                                    view! {
                                        <ItemTooltipContent
                                            item_specs
                                            class:select-text
                                            show_affixes
                                            max_item_level
                                        />
                                    }
                                        .into_any()
                                })
                                .unwrap_or_else(|| {
                                    view! {
                                        <div class=format!(
                                            "flex h-full items-center justify-center text-sm xl:text-base text-gray-400 {}",
                                            empty_label_class.unwrap_or("text-center"),
                                        )>{empty_label}</div>
                                    }
                                        .into_any()
                                })
                        }}
                    </div>
                </div>
            </div>
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
            class="absolute flex top-2 right-2 px-2 py-1 rounded-[4px] items-center border border-zinc-700/80 bg-black/45 hover:bg-black/60"

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
            <span class="text-gray-400 text-xs xl:text-sm">"Compare"</span>
        </div>
    }
}
