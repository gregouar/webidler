use std::{sync::Arc, time::Duration};

use leptos::{html::Div, portal::Portal, prelude::*, web_sys};
use shared::data::{
    area::AreaLevel,
    item::{InventorySortType, ItemSlot, ItemSpecs},
};

use crate::components::{
    accessibility::AccessibilityContext,
    chat::chat_context::ChatContext,
    events::{EventsContext, Key},
    ui::{
        buttons::MenuButton,
        context_menu::{ActionMenuRow, ActionMenuTone, ContextMenu},
        tooltip::DynamicTooltipPosition,
    },
};

use super::{item_card::ItemCard, tooltips::ItemTooltip};

/// An optional item action. Callbacks read live state for the item's current index.
#[derive(Clone, Copy)]
pub struct BagAction {
    pub label: Callback<usize, String>,
    pub on_action: Callback<usize>,
    pub tone: ActionMenuTone,
    pub visible: Option<Callback<usize, bool>>,
    pub disabled: Option<Callback<usize, bool>>,
    /// The first visible action with this flag handles desktop right-clicks.
    pub right_click: bool,
    /// Temporarily cover the item after invoking this action, until its data changes.
    pub pending_duration: Option<Duration>,
}

impl BagAction {
    pub fn new(label: &'static str, on_action: impl Fn(usize) + Send + Sync + 'static) -> Self {
        Self {
            label: Callback::new(move |_| label.to_string()),
            on_action: Callback::new(on_action),
            tone: ActionMenuTone::Success,
            visible: None,
            disabled: None,
            right_click: false,
            pending_duration: None,
        }
    }

    fn is_visible(self, index: usize) -> bool {
        self.visible.is_none_or(|visible| visible.run(index))
    }

    fn is_disabled(self, index: usize) -> bool {
        self.disabled.is_some_and(|disabled| disabled.run(index))
    }
}

impl Default for BagAction {
    fn default() -> Self {
        Self::new("", move |_| {})
    }
}

#[derive(Clone, Default)]
pub struct BagConfig {
    pub items: Signal<Vec<ItemSpecs>>,
    pub capacity: Signal<usize>,
    pub max_item_level: Signal<AreaLevel>,
    pub actions: Vec<BagAction>,
    /// Select directly and open a Cancel-only menu without a tooltip.
    pub on_select: Option<Callback<usize>>,
    pub on_cancel: Option<Callback<usize>>,
    pub overlay: Option<Callback<usize, AnyView>>,
    pub comparable_item: Option<Callback<ItemSlot, Option<Arc<ItemSpecs>>>>, // To allow comparison with equipped item
    pub dimmed: Option<Callback<usize, bool>>,
    pub sell_badge: Option<Callback<usize, bool>>,
    pub show_sell_price: bool,
    /// Share this signal between bags to allow only one open menu in the group.
    pub selected: RwSignal<Option<(&'static str, usize)>>,
    /// Must be unique among bags sharing `selected`; unrelated bags need no ID.
    pub menu_id: &'static str,
    pub context_tooltip_right: bool,
}

impl BagConfig {
    fn right_click_action(&self, index: usize) -> Option<&BagAction> {
        self.actions
            .iter()
            .find(|action| action.right_click && action.is_visible(index))
    }
}

#[component]
pub fn BagGrid(config: BagConfig, compact: bool) -> impl IntoView {
    let capacity = config.capacity;
    let config = Arc::new(config);
    view! {
        <div class=if compact {
            "grid grid-cols-6 xl:grid-cols-8 gap-1 xl:gap-x-3 xl:gap-y-2 px-2 xl:px-3 relative"
        } else {
            "grid grid-cols-8 xl:grid-cols-10 gap-1 xl:gap-x-3 xl:gap-y-2 px-2 xl:px-3 relative"
        }>
            <For each=move || 0..capacity.get() key=|i| *i let(i)>
                <BagItem config=config.clone() item_index=i />
            </For>
        </div>
    }
}

#[component]
fn BagItem(config: Arc<BagConfig>, item_index: usize) -> impl IntoView {
    let events: EventsContext = expect_context();
    let chat: ChatContext = expect_context();
    let accessibility: AccessibilityContext = expect_context();

    let max_item_level = config.max_item_level;
    let show_sell_price = config.show_sell_price;
    let context_tooltip_right = config.context_tooltip_right;
    let selection_key = (config.menu_id, item_index);
    let selected = config.selected;
    let on_select = config.on_select;
    let on_cancel = config.on_cancel;
    let overlay = config.overlay;

    let pending = RwSignal::new(false);
    let maybe_item = Memo::new({
        let config = config.clone();
        move |_| {
            config
                .items
                .with(|items| items.get(item_index).cloned().map(Arc::new))
        }
    });

    let comparable_item = Memo::new({
        let config = config.clone();
        move |_| {
            maybe_item
                .with(|item| item.as_ref().and_then(|item| item.base.slot))
                .and_then(|slot| config.comparable_item.and_then(|compare| compare.run(slot)))
        }
    });
    let dimmed = {
        let config = config.clone();
        move || config.dimmed.is_some_and(|dimmed| dimmed.run(item_index))
    };
    let badge = Signal::derive({
        let config = config.clone();
        move || config.sell_badge.map(|badge| badge.run(item_index))
    });
    let show_menu = move || selected.get() == Some(selection_key);

    Effect::new(move || {
        maybe_item.track();
        pending.set(false);
        if selected.get_untracked() == Some(selection_key) {
            selected.set(None);
        }
    });
    let invoke = Callback::new(move |action: BagAction| {
        if pending.get_untracked()
            || maybe_item.get_untracked().is_none()
            || !action.is_visible(item_index)
            || action.is_disabled(item_index)
        {
            return;
        }

        action.on_action.run(item_index);

        selected.set(None);
        if let Some(duration) = action.pending_duration {
            pending.set(true);
            set_timeout(
                move || {
                    pending.try_set(false);
                },
                duration,
            );
        }
    });
    let item_ref = NodeRef::<Div>::new();

    view! {
        <div class="min-w-0">
            <div node_ref=item_ref class="relative group w-full aspect-[2/3]">
                {move || match maybe_item.get() {
                    Some(item_specs) => {
                        let chat = chat.clone();
                        let config = config.clone();
                        let dimmed = dimmed.clone();
                        view! {
                            <div class="relative w-full h-full overflow-visible">
                                <ItemCard
                                    item_specs=item_specs.clone()
                                    comparable_item_specs=comparable_item.get()
                                    class:brightness-50=dimmed
                                    on:click=move |_| {
                                        if events.key_pressed(Key::Shift) {
                                            chat.link_item(item_specs.clone());
                                        } else if let Some(on_select) = on_select {
                                            on_select.run(item_index);
                                            selected.set(Some(selection_key));
                                        } else {
                                            selected.set(Some(selection_key));
                                        }
                                    }
                                    on:contextmenu={
                                        let config = config.clone();
                                        move |ev| {
                                            ev.prevent_default();
                                            if !accessibility.is_on_mobile()
                                                && let Some(action) = config.right_click_action(item_index)
                                            {
                                                invoke.run(action.clone());
                                            }
                                        }
                                    }
                                    tooltip_position=if context_tooltip_right {
                                        DynamicTooltipPosition::AutoRight
                                    } else {
                                        DynamicTooltipPosition::AutoLeft
                                    }
                                    max_item_level
                                    can_sell=show_sell_price
                                />
                                {overlay
                                    .map(|overlay| {
                                        view! {
                                            <div class="pointer-events-none absolute bottom-1 right-1 z-20 max-w-[calc(100%-0.5rem)]">
                                                {move || overlay.run(item_index)}
                                            </div>
                                        }
                                    })}
                                <Show when=move || badge.get().unwrap_or_default()>
                                    <div class="absolute top-1 right-1 z-20 px-1.5 xl:px-2 py-0.5 text-[10px] xl:text-xs font-black tracking-[0.08em] text-[#ffe0d3] border border-[#8e4538] rounded-[3px] shadow-[0_3px_8px_rgba(0,0,0,0.45),inset_0_1px_0_rgba(255,214,194,0.18)] bg-[linear-gradient(180deg,rgba(230,164,125,0.12),rgba(0,0,0,0.18)),linear-gradient(180deg,rgba(72,28,26,0.98),rgba(35,11,13,1))]">
                                        "SELL"
                                    </div>
                                </Show>
                                <Show when=move || pending.get()>
                                    <div
                                        class="absolute inset-0 z-30 w-full"
                                        style="background: linear-gradient(180deg, rgba(214,177,102,0.04), rgba(0,0,0,0.08)), linear-gradient(135deg, rgba(32,31,36,0.82), rgba(8,8,10,0.92)); box-shadow: inset 0 0 0 1px rgba(108,83,41,0.55), inset 0 0 18px rgba(0,0,0,0.45);"
                                    ></div>
                                </Show>
                                <Show when=show_menu>
                                    {
                                        let config = config.clone();
                                        view! {
                                            <ContextMenu on_close=Callback::new(move |_| {
                                                if selected.get_untracked() == Some(selection_key) {
                                                    selected.set(None);
                                                }
                                            })>
                                                {config
                                                    .actions
                                                    .clone()
                                                    .into_iter()
                                                    .map(|action| {
                                                        view! {
                                                            <Show when=move || action.is_visible(item_index)>
                                                                <ActionMenuRow
                                                                    label_signal=Signal::derive(move || {
                                                                        action.label.run(item_index)
                                                                    })
                                                                    tone=action.tone
                                                                    disabled=Signal::derive(move || {
                                                                        pending.get() || action.is_disabled(item_index)
                                                                    })
                                                                    on_click=move || invoke.run(action.clone())
                                                                />
                                                            </Show>
                                                        }
                                                    })
                                                    .collect_view()}
                                                <ActionMenuRow
                                                    label="Cancel"
                                                    tone=ActionMenuTone::Neutral
                                                    on_click=move || {
                                                        selected.set(None);
                                                        if let Some(on_cancel) = on_cancel {
                                                            on_cancel.run(item_index);
                                                        }
                                                    }
                                                />
                                            </ContextMenu>
                                        }
                                    } <Show when=move || on_select.is_none()>
                                        <BagContextTooltip
                                            item_ref
                                            item_specs=maybe_item.get().unwrap()
                                            max_item_level
                                            right=context_tooltip_right
                                        />
                                    </Show>
                                </Show>
                            </div>
                        }
                            .into_any()
                    }
                    None => view! { <EmptySlot /> }.into_any(),
                }}
            </div>
        </div>
    }
}

#[component]
fn BagContextTooltip(
    item_ref: NodeRef<Div>,
    item_specs: Arc<ItemSpecs>,
    max_item_level: Signal<AreaLevel>,
    right: bool,
) -> impl IntoView {
    view! {
        <Portal>
            {
                let tooltip_ref = NodeRef::<Div>::new();
                let tooltip_size = Memo::new(move |_| {
                    tooltip_ref
                        .get()
                        .map(|tooltip| {
                            let rect = tooltip.get_bounding_client_rect();
                            (rect.width(), rect.height())
                        })
                        .unwrap_or_default()
                });
                let position = move || {
                    let Some(item) = item_ref.get() else {
                        return "display:none;".to_string();
                    };
                    let rect = item.get_bounding_client_rect();
                    let (width, height) = tooltip_size.get();
                    if width == 0.0 {
                        return "left:0px; top:0px; visibility:hidden;".to_string();
                    }
                    let window = web_sys::window().unwrap();
                    let window_height = window.inner_height().unwrap().as_f64().unwrap();
                    let x = if right {
                        let window_width = window.inner_width().unwrap().as_f64().unwrap();
                        rect.right().min(window_width - width)
                    } else {
                        rect.left() - width
                    };
                    format!(
                        "left:{}px; top:{}px;",
                        x.max(0.0),
                        rect.top().min(window_height - height),
                    )
                };
                // Render at the origin before measuring, as in the original bag.
                // Starting beside a right-edge item constrains the tooltip's width.
                view! {
                    <div
                        node_ref=tooltip_ref
                        class="fixed left-0 z-50 transition-opacity duration-150 text-center px-2"
                        style=position
                    >
                        <ItemTooltip item_specs=item_specs.clone() max_item_level />
                    </div>
                }
            }
        </Portal>
    }
}

#[component]
pub fn BagSortButton(
    #[prop(into)] on_sort: Callback<InventorySortType>,
    #[prop(optional, into)] disabled: Signal<bool>,
) -> impl IntoView {
    let last_sort = RwSignal::new(None::<InventorySortType>);
    view! {
        <MenuButton
            disabled
            on:click=move |_| {
                let sort_type = last_sort
                    .get_untracked()
                    .map(InventorySortType::next)
                    .unwrap_or_default();
                last_sort.set(Some(sort_type));
                on_sort.run(sort_type);
            }
        >
            "Sort"
        </MenuButton>
    }
}

#[component]
pub(super) fn EmptySlot(#[prop(optional)] children: Option<Children>) -> impl IntoView {
    view! {
        <div class="relative isolate flex items-center justify-center w-full h-full overflow-clip
        rounded-[4px] xl:rounded-[6px] opacity-80 border border-[#56462f]/80 
        shadow-[0_3px_7px_rgba(0,0,0,0.24),inset_0_1px_0_rgba(214,177,102,0.06),inset_0_-1px_0_rgba(0,0,0,0.38)] 
        bg-[linear-gradient(180deg,rgba(214,177,102,0.03),rgba(0,0,0,0.12)),linear-gradient(135deg,rgba(39,38,44,0.94),rgba(15,15,18,1))]">
            <div class="pointer-events-none absolute inset-[1px] rounded-[3px] xl:rounded-[5px] border border-white/5"></div>
            <div class="relative z-10 flex h-full w-full items-center justify-center p-1">
                {children.map(|children| children())}
            </div>
        </div>
    }
}
