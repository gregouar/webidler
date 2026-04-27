use std::sync::Arc;

use codee::{Decoder, binary::MsgpackSerdeCodec};
use leptos::{
    ev::{mousemove, mouseup},
    prelude::*,
    web_sys::wasm_bindgen::JsCast,
};
use leptos_use::use_resize_observer;

use shared::data::item::ItemSpecs;
use shared_chat::types::{ChatChannel, ChatMessage};

use crate::{
    assets::img_asset,
    components::{
        chat::chat_context::ChatContext,
        events::{EventsContext, Key},
        shared::tooltips::{ItemTooltip, item_tooltip},
        ui::{checkbox::Checkbox, number::format_datetime, tooltip::DynamicTooltipTarget},
    },
};

#[component]
pub fn ChatPanel() -> impl IntoView {
    let chat_context: ChatContext = expect_context();
    let events_context: EventsContext = expect_context();

    let panel_ref = NodeRef::new();
    let position = RwSignal::new((50i32, 50i32)); // bottom, left

    let clamp_panel = move || {
        let (bottom, left) = position.get_untracked();

        let win = window();
        let height = win.inner_height().unwrap().as_f64().unwrap() as i32;
        let width = win.inner_width().unwrap().as_f64().unwrap() as i32;

        let panel: web_sys::HtmlDivElement = panel_ref.get().unwrap();
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
            .iter_rev()
            .find(|m| matches!(m.channel, ChatChannel::Whisper(_)) || selected.contains(&m.channel))
            .cloned()
    };

    // TODO: Do better than that...
    let filtered_messages = move || {
        let selected = chat_context.selected_channels.get();
        let mut messages = chat_context
            .messages
            .read()
            .iter()
            .filter(|m| {
                matches!(m.channel, ChatChannel::Whisper(_)) || selected.contains(&m.channel)
            })
            .cloned()
            .collect::<Vec<_>>();
        messages.sort_by_key(|message| message.sent_at);
        messages
    };

    let dropdown_open = RwSignal::new(false);

    let send_message = move || {
        let content = input_value.get();
        if content.trim().is_empty() && chat_context.linked_item.read().is_none() {
            return;
        }

        chat_context.send.run(content);

        input_value.set(String::new());
    };

    let messages_node = NodeRef::<leptos::html::Div>::new();
    Effect::new(move || {
        let _ = chat_context.messages.read();
        if let Some(el) = messages_node.get()
            && let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>()
            && is_near_bottom(&html_el)
        {
            html_el.set_scroll_top(html_el.scroll_height());
        }
    });

    Effect::new(move || {
        if !chat_context.minimized.get()
            && chat_context.opened.get()
            && let Some(el) = messages_node.get()
            && let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>()
        {
            html_el.set_scroll_top(html_el.scroll_height());
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
            class="fixed z-50 select-none text-left"
            style=move || {
                let (bottom, left) = position.get();
                format!("bottom:{}px; left:{}px;", bottom, left)
            }
            node_ref=panel_ref

            class:hidden=move || !chat_context.opened.get()
        >
            <div class="w-[420px] bg-zinc-900/80 backdrop-blur border border-zinc-700 text-sm text-gray-200 flex flex-col shadow-xl">

                // Header (drag handle)
                <div
                    class="flex items-center justify-between px-4 py-2 border-b border-zinc-700 bg-zinc-800/80 cursor-move"
                    on:mousedown=start_drag
                >
                    <div class="flex gap-4 items-center">
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

                    <div class="flex gap-3 text-gray-400">
                        <button
                            class="hover:text-white"
                            on:click=move |_| { chat_context.minimized.update(|m| *m = !*m) }
                        >
                            {move || { if chat_context.minimized.get() { "▼" } else { "—" } }}
                        </button>

                        <button
                            class="hover:text-red-400"
                            on:click=move |_| chat_context.opened.set(false)
                        >
                            "✕"
                        </button>
                    </div>
                </div>

                {move || {
                    if chat_context.minimized.get() {
                        view! {
                            <div
                                class="px-4 py-2 bg-zinc-900/70 text-[13px] text-gray-400 text-ellipsis cursor-pointer"
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
                                class="flex-1 overflow-y-auto px-4 py-3 space-y-2 bg-zinc-900/70 max-h-[320px]
                                text-wrap wrap-break-word"
                                node_ref=messages_node
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
                            <div class="border-t border-zinc-700 bg-zinc-900/80">
                                <div class="flex items-stretch">

                                    // Channel selector
                                    <div class="relative">
                                        <button
                                            class="h-full px-3 text-sm border-r border-zinc-700 bg-zinc-800/80 hover:bg-zinc-700/80 flex items-center gap-2"
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
                                                            class="w-full text-left px-3 py-2 hover:bg-zinc-800 text-amber-400"
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
                                                            class="w-full text-left px-3 py-2 hover:bg-zinc-800 text-emerald-400"
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

                                    <div class="flex-1 flex flex-col">
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
                                            class=" resize-none px-3 py-2 text-gray-200 bg-zinc-900/80 focus:outline-none focus:ring-1 focus:ring-amber-500 z-2"
                                            rows="2"
                                            maxlength="200"
                                            prop:value=move || input_value.get()
                                            on:input=move |ev| {
                                                input_value.set(event_target_value(&ev));
                                            }
                                            on:keydown=move |ev| {
                                                if ev.key() == "Enter" && !ev.shift_key() {
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

            </div>
        </div>
    }
}

#[component]
fn ChatMessageRow(msg: ChatMessage) -> impl IntoView {
    let chat_context: ChatContext = expect_context();

    view! {
        <div class="text-sm flex" title=format!("Sent at {}", format_datetime(msg.sent_at))>
            {msg
                .chat_badge
                .as_ref()
                .map(|chat_badge| {
                    view! {
                        <img
                            src=img_asset(&format!("badges/{}", chat_badge))
                            alt="Badge"
                            class="h-[32px] mr-1 aspect-square"
                        />
                    }
                })}
            <span
                class=move || { format!("cursor-pointer {}", channel_color(msg.channel)) }
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
                {author_str(&msg)}
            </span>
            <span class="text-gray-500">": "</span>
            {msg
                .linked_item
                .and_then(|item_data| MsgpackSerdeCodec::decode(&item_data.into_inner()).ok())
                .map(|item_specs: ItemSpecs| {
                    view! { <ChatItem item_specs=Arc::new(item_specs) /> }
                })}
            <span class="text-gray-200 select-text">{msg.content}</span>
        </div>
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
                "font-bold {}",
                item_tooltip::name_color_rarity(item_specs.modifiers.rarity),
            )>"<" {item_specs.modifiers.name.clone()} "> "</span>
        </DynamicTooltipTarget>
    }
}

fn author_str(msg: &ChatMessage) -> String {
    let chat_context: ChatContext = expect_context();

    if let ChatChannel::System = msg.channel {
        "[System]".into()
    } else if let ChatChannel::Whisper(_) = msg.channel
        && msg.user_id == chat_context.user_id.get()
    {
        channel_str(msg.channel)
    } else {
        msg.username.clone().unwrap_or_default()
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
