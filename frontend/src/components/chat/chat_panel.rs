use std::collections::HashSet;

use leptos::{
    ev::{mousemove, mouseup},
    prelude::*,
    web_sys::wasm_bindgen::JsCast,
};

use shared_chat::types::ChatChannel;

use crate::components::{chat::chat_context::ChatContext, ui::checkbox::Checkbox};

#[component]
pub fn ChatPanel(open: RwSignal<bool>) -> impl IntoView {
    let chat_context: ChatContext = expect_context();

    let minimized = RwSignal::new(false);

    // Drag state
    let dragging = RwSignal::new(false);
    let position = RwSignal::new((200i32, 200i32)); // top, left
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
            let (start_top, start_left) = drag_start_position.get();

            let dx = ev.screen_x() - start_mx;
            let dy = ev.screen_y() - start_my;

            let new_top = start_top + dy;
            let new_left = start_left + dx;

            // TODO: Compute actual size of chat panel?

            // Clamp AFTER computing absolute value
            let win = window();
            let height = win.inner_height().unwrap().as_f64().unwrap() as i32;
            let width = win.inner_width().unwrap().as_f64().unwrap() as i32;

            let clamped_top = new_top.clamp(0, height - 50);
            let clamped_left = new_left.clamp(0, width - 300);

            position.set((clamped_top, clamped_left));
        });

        let up_listener = window_event_listener(mouseup, move |_| {
            dragging.set(false);
        });

        drop(move_listener);
        drop(up_listener);
    };

    let selected_channels = RwSignal::new({
        let mut set = HashSet::new();
        set.insert(ChatChannel::Global);
        set.insert(ChatChannel::System);
        set
    });

    let input_value = RwSignal::new(String::new());

    let last_visible_message = move || {
        let selected = selected_channels.get();
        chat_context
            .messages
            .read()
            .iter_rev()
            .find(|m| selected.contains(&m.channel))
            .cloned()
    };

    // TODO: Do better than that...
    let filtered_messages = move || {
        let selected = selected_channels.get();
        chat_context
            .messages
            .read()
            .iter()
            .filter(|m| selected.contains(&m.channel))
            .cloned()
            .collect::<Vec<_>>()
    };

    let write_channel = RwSignal::new(ChatChannel::Global);
    let dropdown_open = RwSignal::new(false);

    let send_message = move || {
        let content = input_value.get();
        if content.trim().is_empty() {
            return;
        }

        chat_context
            .send
            .run((write_channel.get_untracked(), content));

        input_value.set(String::new());
    };

    let messages_node = NodeRef::<leptos::html::Div>::new();
    Effect::new(move || {
        let _ = chat_context.messages.read();
        if let Some(el) = messages_node.get() {
            if let Ok(html_el) = el.dyn_into::<web_sys::HtmlElement>() {
                if is_near_bottom(&html_el) {
                    html_el.set_scroll_top(html_el.scroll_height());
                }
            }
        }
    });

    // TODO: Split in components
    view! {
        {move || {
            if !open.get() {
                ().into_any()
            } else {
                let (top, left) = position.get();
                view! {
                    <div
                        class="fixed z-50 select-none text-left"
                        style=format!("top:{}px; left:{}px;", top, left)
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
                                                            selected_channels.write().insert(channel);
                                                        } else {
                                                            selected_channels.write().remove(&channel);
                                                        }
                                                    }
                                                    checked=Signal::derive(move || {
                                                        selected_channels.get().contains(&channel)
                                                    })
                                                />
                                            }
                                        })
                                        .collect::<Vec<_>>()}
                                </div>

                                <div class="flex gap-3 text-gray-400">
                                    <button
                                        class="hover:text-white"
                                        on:click=move |_| minimized.update(|m| *m = !*m)
                                    >
                                        {move || if minimized.get() { "▼" } else { "—" }}
                                    </button>

                                    <button
                                        class="hover:text-red-400"
                                        on:click=move |_| open.set(false)
                                    >
                                        "✕"
                                    </button>
                                </div>
                            </div>

                            {move || {
                                if minimized.get() {
                                    view! {
                                        <div
                                            class="px-4 py-2 bg-zinc-900/70 text-[13px] text-gray-400 truncate cursor-pointer"
                                            on:click=move |_| minimized.set(false)
                                        >
                                            {move || {
                                                if let Some(msg) = last_visible_message() {
                                                    format!(
                                                        "{}: {}",
                                                        msg.user_name.unwrap_or_default(),
                                                        msg.content.into_inner(),
                                                    )
                                                } else {
                                                    "No messages".into()
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
                                                    view! {
                                                        <div class="text-[13px] leading-snug">
                                                            <span class=move || channel_color(
                                                                msg.channel,
                                                            )>{msg.user_name.clone()}</span>
                                                            <span class="text-gray-500">": "</span>
                                                            <span class="text-gray-200">
                                                                {msg.content.into_inner()}
                                                            </span>
                                                        </div>
                                                    }
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
                                                            write_channel.get(),
                                                        )>{move || channel_str(write_channel.get())}</span>
                                                    // <span class="text-gray-500">"▾"</span>
                                                    </button>

                                                    {move || {
                                                        if dropdown_open.get() {
                                                            view! {
                                                                <div class="absolute bottom-full left-0 w-28 bg-zinc-900 border border-zinc-700 shadow-lg text-sm">

                                                                    <button
                                                                        class="w-full text-left px-3 py-2 hover:bg-zinc-800 text-amber-400"
                                                                        on:click=move |_| {
                                                                            write_channel.set(ChatChannel::Global);
                                                                            selected_channels.write().insert(ChatChannel::Global);
                                                                            dropdown_open.set(false);
                                                                        }
                                                                    >
                                                                        "Global"
                                                                    </button>

                                                                    <button
                                                                        class="w-full text-left px-3 py-2 hover:bg-zinc-800 text-emerald-400"
                                                                        on:click=move |_| {
                                                                            write_channel.set(ChatChannel::Trade);
                                                                            selected_channels.write().insert(ChatChannel::Trade);
                                                                            dropdown_open.set(false);
                                                                        }
                                                                    >
                                                                        "Trade"
                                                                    </button>

                                                                </div>
                                                            }
                                                                .into_any()
                                                        } else {
                                                            ().into_any()
                                                        }
                                                    }}
                                                </div>

                                                // Textarea
                                                <textarea
                                                    class="flex-1 resize-none px-3 py-2 text-gray-200 bg-zinc-900/80 focus:outline-none focus:ring-1 focus:ring-amber-500 z-2"
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
                                                />
                                            </div>
                                        </div>
                                    }
                                        .into_any()
                                }
                            }}

                        </div>
                    </div>
                }
                    .into_any()
            }
        }}
    }
}

fn channel_str(channel: ChatChannel) -> &'static str {
    match channel {
        ChatChannel::System => "System",
        ChatChannel::Global => "Global",
        ChatChannel::Trade => "Trade",
    }
}

fn channel_color(channel: ChatChannel) -> &'static str {
    match channel {
        ChatChannel::Global => "text-amber-400",
        ChatChannel::Trade => "text-emerald-400",
        ChatChannel::System => "text-fuchsia-400",
    }
}

fn is_near_bottom(el: &web_sys::HtmlElement) -> bool {
    let scroll_height = el.scroll_height() as f64;
    let scroll_top = el.scroll_top() as f64;
    let client_height = el.client_height() as f64;

    (scroll_height - scroll_top - client_height) < 80.0
}
