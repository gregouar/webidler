use std::{sync::Arc, time::Duration};

use codee::{Decoder, binary::MsgpackSerdeCodec};
use leptos::{
    ev::{mousemove, mouseup},
    prelude::*,
    web_sys::wasm_bindgen::JsCast,
};
use leptos_use::use_resize_observer;

use shared::data::{cosmetics::CosmeticType, item::ItemSpecs, user::UserCharacterId};
use shared_chat::types::{ChatChannel, ChatMessage, UserId};

use crate::{
    assets::img_asset,
    components::{
        chat::chat_context::ChatContext,
        data_context::DataContext,
        events::{EventsContext, Key, keyboard_event_key},
        shared::tooltips::{ItemTooltip, item_tooltip},
        ui::{
            card::Card,
            checkbox::Checkbox,
            number::format_datetime,
            tooltip::{DynamicTooltipTarget, StaticTooltip, StaticTooltipPosition},
        },
    },
};

#[component]
pub fn ChatPanel(
    #[prop(optional)] character_id: Option<UserCharacterId>,
    #[prop(optional)] pinnable: bool,
) -> impl IntoView {
    let chat_context: ChatContext = expect_context();
    let events_context: EventsContext = expect_context();

    let pinned = move || pinnable && chat_context.pinned.get();
    let panel_ref = NodeRef::new();
    let position = RwSignal::new((50i32, 50i32)); // bottom, left

    let clamp_panel = move || {
        if pinned() {
            return;
        }
        let (bottom, left) = position.get_untracked();

        let win = window();
        let height = win.inner_height().unwrap().as_f64().unwrap() as i32;
        let width = win.inner_width().unwrap().as_f64().unwrap() as i32;

        let Some(panel): Option<web_sys::HtmlDivElement> = panel_ref.get() else {
            return;
        };
        let rect = panel.get_bounding_client_rect();
        let panel_width = rect.width() as i32;
        let panel_height = rect.height() as i32;

        let clamped_bottom = bottom.clamp(0, (height - panel_height).max(0));
        let clamped_left = left.clamp(0, (width - panel_width).max(0));

        position.set((clamped_bottom, clamped_left));
    };

    // Drag state
    let dragging = RwSignal::new(false);
    let drag_start_mouse = RwSignal::new((0i32, 0i32));
    let drag_start_position = RwSignal::new((0i32, 0i32));
    let start_drag = move |ev: leptos::ev::MouseEvent| {
        if pinned() {
            return;
        }
        dragging.set(true);

        drag_start_mouse.set((ev.screen_x(), ev.screen_y()));
        drag_start_position.set(position.get());

        let move_listener = window_event_listener(mousemove, move |ev| {
            if !dragging.try_get().unwrap_or_default() {
                return;
            }

            let (start_mx, start_my) = drag_start_mouse.get();
            let (start_bottom, start_left) = drag_start_position.get();

            let dx = ev.screen_x() - start_mx;
            let dy = ev.screen_y() - start_my;

            position.set(((start_bottom - dy), (start_left + dx)));
            clamp_panel()
        });

        let up_listener = window_event_listener(mouseup, move |_| {
            dragging.set(false);
        });

        drop(move_listener);
        drop(up_listener);
    };

    use_resize_observer(panel_ref, move |_, _| {
        clamp_panel();
    });

    let input_value = RwSignal::new(String::new());

    let last_visible_message = move || {
        let selected = chat_context.selected_channels.get();
        chat_context
            .messages
            .read()
            .iter()
            .rev()
            .find(|m| matches!(m.channel, ChatChannel::Whisper(_)) || selected.contains(&m.channel))
            .cloned()
    };

    // TODO: Do better than that...
    let filtered_messages = move || {
        let selected = chat_context.selected_channels.get();
        chat_context
            .messages
            .read()
            .iter()
            .filter(|m| {
                matches!(m.channel, ChatChannel::Whisper(_)) || selected.contains(&m.channel)
            })
            .cloned()
            .collect::<Vec<_>>()
    };

    let dropdown_open = RwSignal::new(false);

    let send_message = move || {
        let content = input_value.get();
        if content.trim().is_empty() && chat_context.linked_item.read().is_none() {
            return;
        }

        chat_context.send.run((content, character_id));

        input_value.set(String::new());
    };

    let messages_node = NodeRef::<leptos::html::Div>::new();
    let should_stick_to_bottom = RwSignal::new(true);

    Effect::new(move || {
        let _ = chat_context.messages.read();
        let _ = chat_context.selected_channels.read();
        if should_stick_to_bottom.get_untracked()
            && !chat_context.minimized.get_untracked()
            && chat_context.opened.get_untracked()
        {
            scroll_messages_to_bottom_after_render(messages_node);
        }
    });

    Effect::new(move || {
        if !chat_context.minimized.get() && chat_context.opened.get() {
            should_stick_to_bottom.set(true);
            scroll_messages_to_bottom_after_render(messages_node);
        }
    });

    let text_area_ref: NodeRef<leptos::html::Textarea> = NodeRef::new();
    // Effect::new(move || {
    //     if chat_context.opened.get()
    //         && let Some(text_area) = text_area_ref.get_untracked()
    //     {
    //         text_area.focus().unwrap();
    //         text_area.select();
    //     }
    // });

    Effect::new(move || {
        if chat_context.linked_item.read().is_some()
            && let Some(text_area) = text_area_ref.get_untracked()
        {
            text_area.focus().unwrap();
            text_area.select();
        }
    });

    Effect::new(move || {
        if events_context.key_pressed(Key::Enter) {
            chat_context.opened.set(true);
            chat_context.minimized.set(false);
            if let Some(text_area) = text_area_ref.get_untracked() {
                text_area.focus().unwrap();
                text_area.select();
            }
        }
    });

    // TODO: Split in components
    view! {
        <div
            class=move || {
                if pinned() {
                    "min-h-0 w-full shrink-0 select-none text-left"
                } else {
                    "fixed z-50 w-[420px] max-w-[100vw] select-none text-left"
                }
            }
            style=move || {
                if pinned() {
                    return if chat_context.minimized.get() {
                        String::new()
                    } else {
                        "height:25%;max-height:300px;".to_owned()
                    };
                }
                let (bottom, left) = position.get();
                format!("bottom:{}px; left:{}px;", bottom, left)
            }
            node_ref=panel_ref

            class:hidden=move || !chat_context.opened.get()
        >
            <ChatCard pinned=Signal::derive(pinned)>

                // Header (drag handle)
                <div
                    class="flex shrink-0 flex-wrap items-center justify-between gap-2 px-3 py-2 border-b border-zinc-700 bg-zinc-800/80"
                    class:cursor-move=move || !pinned()
                    style:background-color=move || {
                        if pinned() { "" } else { "rgb(39 39 42 / 0.85)" }
                    }
                    on:mousedown=start_drag
                >
                    <div class="flex flex-wrap gap-2 items-center">
                        {[ChatChannel::Global, ChatChannel::Trade, ChatChannel::System]
                            .into_iter()
                            .map(move |channel| {
                                view! {
                                    <Checkbox
                                        label=channel_str(channel)
                                        on_change=move |value| {
                                            if value {
                                                chat_context.selected_channels.write().insert(channel);
                                            } else {
                                                chat_context.selected_channels.write().remove(&channel);
                                            }
                                        }
                                        checked=Signal::derive(move || {
                                            chat_context.selected_channels.get().contains(&channel)
                                        })
                                    />
                                }
                            })
                            .collect::<Vec<_>>()}
                    </div>

                    <div
                        class="flex gap-3 text-zinc-400"
                        on:mousedown=move |ev| ev.stop_propagation()
                    >
                        <button
                            class="hover:text-white"
                            on:click=move |_| { chat_context.minimized.update(|m| *m = !*m) }
                            title=move || {
                                if chat_context.minimized.get() {
                                    "Expand chat"
                                } else {
                                    "Reduce chat"
                                }
                            }
                        >
                            {move || { if chat_context.minimized.get() { "▼" } else { "—" } }}
                        </button>
                        <Show when=move || pinnable>
                            <button
                                class="flex items-center hover:text-white"
                                title=move || {
                                    if pinned() { "Unpin chat" } else { "Pin chat below player" }
                                }
                                aria-label=move || {
                                    if pinned() { "Unpin chat" } else { "Pin chat below player" }
                                }
                                aria-pressed=move || pinned().to_string()
                                on:click=move |_| chat_context.set_pinned.set(Some(!pinned()))
                            >
                                <svg
                                    class="h-4 w-4"
                                    viewBox="0 0 24 24"
                                    fill="none"
                                    stroke="currentColor"
                                    stroke-width="1.75"
                                    stroke-linecap="round"
                                    stroke-linejoin="round"
                                    aria-hidden="true"
                                >
                                    <path d="M16 3H8l1 6-3 3v3h12v-3l-3-3 1-6ZM12 15v6" />
                                    <Show when=pinned>
                                        <path d="m3 3 18 18" />
                                    </Show>
                                </svg>
                            </button>
                        </Show>
                        <button
                            class="hover:text-red-400"
                            on:click=move |_| chat_context.opened.set(false)
                            title="Close chat"
                        >
                            "✕"
                        </button>
                    </div>
                </div>

                {move || {
                    if chat_context.minimized.get() {
                        view! {
                            <div
                                class="px-3 py-2 bg-zinc-900/70 text-[13px] text-zinc-400 overflow-hidden max-h-14 cursor-pointer"
                                style:background-color=move || {
                                    if pinned() { "" } else { "rgb(24 24 27 / 0.9)" }
                                }
                                on:click=move |_| chat_context.minimized.set(false)
                            >
                                {move || {
                                    if let Some(msg) = last_visible_message() {
                                        view! { <ChatMessageRow msg /> }.into_any()
                                    } else {
                                        "No messages".into_any()
                                    }
                                }}
                            </div>
                        }
                            .into_any()
                    } else {
                        view! {
                            // Messages
                            <div
                                class="flex-1 min-h-0 overflow-y-auto px-4 py-3 space-y-2 bg-zinc-900/70 max-h-[320px]
                                text-wrap wrap-break-word"
                                style:background-color=move || {
                                    if pinned() { "" } else { "rgb(24 24 27 / 0.9)" }
                                }
                                node_ref=messages_node
                                on:scroll=move |_| {
                                    if let Some(el) = messages_node.get()
                                        && let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>()
                                    {
                                        should_stick_to_bottom.set(is_near_bottom(&html_el));
                                    }
                                }
                            >
                                <For
                                    each=filtered_messages
                                    key=|msg| (msg.sent_at, msg.user_id)
                                    children=move |msg| {
                                        view! { <ChatMessageRow msg /> }
                                    }
                                />
                            </div>

                            // Input
                            <div
                                class="shrink-0 border-t border-zinc-700 bg-zinc-900/80"
                                style:background-color=move || {
                                    if pinned() { "" } else { "rgb(24 24 27 / 0.85)" }
                                }
                            >
                                <div class="flex items-stretch">

                                    // Channel selector
                                    <div class="relative">
                                        <button
                                            class=move || {
                                                format!(
                                                    "h-full px-3 text-sm border-r border-zinc-700 hover:bg-zinc-600/80 active:bg-zinc-500/80 active:shadow-inner transition-colors flex items-center gap-2 {}",
                                                    if dropdown_open.get() {
                                                        "bg-zinc-700/80"
                                                    } else if pinned() {
                                                        "bg-zinc-800/80"
                                                    } else {
                                                        "bg-transparent"
                                                    },
                                                )
                                            }
                                            aria-expanded=move || dropdown_open.get().to_string()
                                            on:click=move |_| dropdown_open.update(|o| *o = !*o)
                                        >
                                            <span class=move || channel_color(
                                                chat_context.write_channel.get(),
                                            )>
                                                {move || channel_str(chat_context.write_channel.get())}
                                            </span>
                                        // <span class="text-gray-500">"▾"</span>
                                        </button>

                                        {move || {
                                            if dropdown_open.get() {
                                                view! {
                                                    <div class="absolute bottom-full left-0 w-28 bg-zinc-900 border border-zinc-700 shadow-lg text-sm">

                                                        <button
                                                            class="w-full text-left px-3 py-2 hover:bg-zinc-800 active:bg-zinc-700 active:shadow-inner transition-colors text-amber-400"
                                                            on:click=move |_| {
                                                                chat_context.write_channel.set(ChatChannel::Global);
                                                                chat_context
                                                                    .selected_channels
                                                                    .write()
                                                                    .insert(ChatChannel::Global);
                                                                dropdown_open.set(false);
                                                            }
                                                        >
                                                            {channel_str(ChatChannel::Global)}
                                                        </button>

                                                        <button
                                                            class="w-full text-left px-3 py-2 hover:bg-zinc-800 active:bg-zinc-700 active:shadow-inner transition-colors text-emerald-400"
                                                            on:click=move |_| {
                                                                chat_context.write_channel.set(ChatChannel::Trade);
                                                                chat_context
                                                                    .selected_channels
                                                                    .write()
                                                                    .insert(ChatChannel::Trade);
                                                                dropdown_open.set(false);
                                                            }
                                                        >
                                                            {channel_str(ChatChannel::Trade)}
                                                        </button>

                                                    </div>
                                                }
                                                    .into_any()
                                            } else {
                                                ().into_any()
                                            }
                                        }}
                                    </div>

                                    <div class="min-w-0 flex-1 flex flex-col">
                                        // Textarea
                                        {chat_context
                                            .linked_item
                                            .get()
                                            .map(|item_specs| {
                                                view! {
                                                    <span class="flex px-3 gap-1">
                                                        <button
                                                            class="hover:text-red-400"
                                                            on:click=move |_| chat_context.linked_item.set(None)
                                                        >
                                                            "✕"
                                                        </button>
                                                        <ChatItem item_specs />
                                                    </span>
                                                }
                                            })}
                                        <textarea
                                            class="w-full min-w-0 resize-none px-3 py-2 text-gray-200 bg-zinc-900/80 focus:outline-none z-2"
                                            style:background-color=move || {
                                                if pinned() { "" } else { "transparent" }
                                            }
                                            rows="2"
                                            maxlength="200"
                                            prop:value=move || input_value.get()
                                            on:input=move |ev| {
                                                input_value.set(event_target_value(&ev));
                                            }
                                            on:keydown=move |ev| {
                                                if keyboard_event_key(&ev).as_deref() == Some("Enter")
                                                    && !ev.shift_key()
                                                {
                                                    ev.prevent_default();
                                                    send_message();
                                                }
                                            }
                                            placeholder="Type message..."
                                            node_ref=text_area_ref
                                        />
                                    </div>
                                </div>
                            </div>
                        }
                            .into_any()
                    }
                }}

            </ChatCard>
        </div>
    }
}

#[allow(clippy::unused_unit)]
#[component]
fn ChatCard(pinned: Signal<bool>, children: Children) -> impl IntoView {
    view! {
        <div class="relative flex h-full min-h-0 flex-col text-sm text-gray-200">
            <Show when=move || pinned.get()>
                <div class="pointer-events-none absolute inset-0" aria-hidden="true">
                    <Card class="h-full" pad=false gap=false>
                        {()}
                    </Card>
                </div>
            </Show>
            // Keep the content mounted when pinning so focus and scroll position survive.
            <div class=move || {
                if pinned.get() {
                    "relative z-10 flex min-h-0 flex-1 flex-col m-[2px] overflow-hidden clip-octagon"
                } else {
                    "relative flex min-h-0 flex-1 flex-col border border-zinc-700/50 shadow-xl"
                }
            }>{children()}</div>
        </div>
    }
}

#[component]
fn ChatMessageRow(msg: ChatMessage) -> impl IntoView {
    let chat_context: ChatContext = expect_context();
    let sent_title = format!("Sent at {}", format_datetime(msg.sent_at));

    view! {
        <div class="group/message flex items-start gap-1.5 text-sm leading-[1.35]">
            {msg
                .chat_badge
                .as_ref()
                .map(|badge| {
                    view! { <ChatBadge badge=badge.clone() /> }
                })} <p class="min-w-0 flex-1 text-gray-200 select-text">
                <span
                    class=move || {
                        format!(
                            "cursor-pointer font-medium transition-colors hover:brightness-125 {}",
                            channel_color(msg.channel),
                        )
                    }
                    title=sent_title.clone()
                    on:click=move |_| {
                        if let ChatChannel::Whisper(_) = msg.channel
                            && msg.user_id == chat_context.user_id.get()
                        {
                            chat_context.write_channel.set(msg.channel)
                        } else if let Some(user_id) = msg.user_id {
                            chat_context.write_channel.set(ChatChannel::Whisper(user_id))
                        }
                    }
                >
                    <Author
                        channel=msg.channel
                        user_id=msg.user_id
                        username=msg.username.clone()
                        character_name=msg.character_name.clone()
                        character_title=msg.character_title.clone()
                    />

                </span>
                <span class="text-gray-500 select-none">": "</span>
                {msg
                    .linked_item
                    .and_then(|item_data| MsgpackSerdeCodec::decode(&item_data.into_inner()).ok())
                    .map(|item_specs: ItemSpecs| {
                        view! { <ChatItem item_specs=Arc::new(item_specs) /> }
                    })}
                <span title=sent_title>{msg.content}</span>
            </p>
        </div>
    }
}

#[component]
fn ChatBadge(badge: String) -> impl IntoView {
    let data_context = expect_context::<DataContext>();
    let badge_specs = Memo::new(move |_| {
        data_context
            .cosmetics_specs
            .read()
            .get(&badge)
            .and_then(|cosmetic| match cosmetic {
                CosmeticType::Badge(specs) => Some(specs.clone()),
                _ => None,
            })
    });

    move || {
        badge_specs.get().map(|specs| {
            let src = img_asset(&specs.icon);
            let alt = specs.name.clone();
            let badge_title = specs.name;
            let badge_description = specs.description;
            let tooltip = move || {
                view! {
                    <div class="flex flex-col xl:space-y-1 max-w-[20vw] whitespace-normal">
                        <div class="font-semibold text-white">{badge_title.clone()}</div>
                        <div class="text-sm text-zinc-300">{badge_description.clone()}</div>
                    </div>
                }
            };

            view! {
                <div class="shrink-0">
                    <StaticTooltip position=StaticTooltipPosition::Right tooltip>
                        <img src=src alt=alt class="h-[32px] mr-1 aspect-square" />
                    </StaticTooltip>
                </div>
            }
        })
    }
}

#[component]
fn ChatItem(item_specs: Arc<ItemSpecs>) -> impl IntoView {
    let events_context: EventsContext = expect_context();
    let show_affixes = Memo::new(move |_| events_context.key_pressed(Key::Alt));
    let tooltip = {
        let item_specs = item_specs.clone();
        move || {
            let item_specs = item_specs.clone();
            let show_affixes = show_affixes.get();
            // TODO: Compare? Max Item Level?
            view! {
                <div class="flex gap-1 xl:gap-2">
                    <ItemTooltip item_specs show_affixes />
                </div>
            }
            .into_any()
        }
    };

    view! {
        <DynamicTooltipTarget content=tooltip>
            <span class=format!(
                "mr-1 inline-flex select-none align-baseline items-center rounded border border-current/25 bg-zinc-950/45 px-1.5 py-[1px] font-semibold leading-[1.25] shadow-[inset_0_1px_0_rgba(255,255,255,0.06)] transition-colors hover:bg-zinc-800/80 {}",
                item_tooltip::name_color_rarity(item_specs.modifiers.rarity),
            )>{item_specs.modifiers.name.clone()}</span>
        </DynamicTooltipTarget>
    }
}

#[component]
fn Author(
    channel: ChatChannel,
    user_id: Option<UserId>,
    username: Option<String>,
    character_name: Option<String>,
    character_title: Option<String>,
) -> impl IntoView {
    let chat_context: ChatContext = expect_context();
    let data_context: DataContext = expect_context();

    if let ChatChannel::System = channel {
        "[System]".into_any()
    } else if let ChatChannel::Whisper(_) = channel
        && user_id == chat_context.user_id.get()
    {
        channel_str(channel).into_any()
    } else {
        match (&username, &character_name) {
            (Some(username), Some(character_name)) => {
                let title = character_title.as_ref().and_then(|title| {
                    data_context
                        .cosmetics_specs
                        .read()
                        .get(title)
                        .and_then(|cosmetic| match cosmetic {
                            CosmeticType::Title(title) => Some(title.clone()),
                            _ => None,
                        })
                });

                title
                    .map(|title| {
                        view! {
                            {username.clone()}
                            " ["
                            {character_name.clone()}
                            " — "
                            <span class="italic">{title}</span>
                            "]"
                        }
                        .into_any()
                    })
                    .unwrap_or_else(|| {
                        view! {
                            {username.clone()}
                            " ["
                            {character_name.clone()}
                            "]"
                        }
                        .into_any()
                    })
            }
            (Some(username), None) => username.clone().into_any(),
            _ => String::new().into_any(),
        }
    }
}

fn channel_str(channel: ChatChannel) -> String {
    let chat_context: ChatContext = expect_context();

    match channel {
        ChatChannel::System => "System".into(),
        ChatChannel::Global => "Global".into(),
        ChatChannel::Trade => "Trade".into(),
        ChatChannel::Whisper(user_id) => chat_context
            .users_map
            .read_untracked()
            .get(&user_id)
            .map(|username| format!("@{username}"))
            .unwrap_or("Whisper".into()),
    }
}

fn channel_color(channel: ChatChannel) -> &'static str {
    match channel {
        ChatChannel::Global => "text-amber-400",
        ChatChannel::Trade => "text-emerald-400",
        ChatChannel::System => "text-fuchsia-400",
        ChatChannel::Whisper(_) => "text-cyan-400",
    }
}

fn is_near_bottom(el: &web_sys::HtmlElement) -> bool {
    let scroll_height = el.scroll_height() as f64;
    let scroll_top = el.scroll_top() as f64;
    let client_height = el.client_height() as f64;

    (scroll_height - scroll_top - client_height) < 80.0
}

fn scroll_messages_to_bottom_after_render(messages_node: NodeRef<leptos::html::Div>) {
    set_timeout(
        move || {
            if let Some(el) = messages_node.get()
                && let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>()
            {
                scroll_to_bottom(&html_el);
            }
        },
        Duration::from_millis(0),
    );
}

fn scroll_to_bottom(el: &web_sys::HtmlElement) {
    el.set_scroll_top(el.scroll_height());
}
