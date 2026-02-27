use std::collections::HashSet;

use leptos::ev::{mousemove, mouseup};
use leptos::prelude::*;

use shared::messages::chat::ChatChannel;

#[derive(Clone)]
struct ChatMessage {
    id: u64,
    channel: ChatChannel,
    author: String,
    content: String,
}

#[component]
pub fn ChatPanel(open: RwSignal<bool>) -> impl IntoView {
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
            if !dragging.get() {
                return;
            }

            let (start_mx, start_my) = drag_start_mouse.get();
            let (start_top, start_left) = drag_start_position.get();

            let dx = ev.screen_x() - start_mx;
            let dy = ev.screen_y() - start_my;

            let new_top = start_top + dy;
            let new_left = start_left + dx;

            // TODO: Compute actual size of chat panel

            // Clamp AFTER computing absolute value
            let win = window();
            let height = win.inner_height().unwrap().as_f64().unwrap() as i32;
            let width = win.inner_width().unwrap().as_f64().unwrap() as i32;

            let clamped_top = new_top.clamp(0, height - 100);
            let clamped_left = new_left.clamp(0, width - 200);

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

    let messages = RwSignal::new(vec![
        ChatMessage {
            id: 1,
            channel: ChatChannel::System,
            author: "System".into(),
            content: "World event starting in 2 minutes.".into(),
        },
        ChatMessage {
            id: 2,
            channel: ChatChannel::Global,
            author: "Nyx".into(),
            content: "Anyone pushing wave 200?".into(),
        },
        ChatMessage {
            id: 3,
            channel: ChatChannel::Trade,
            author: "Valen".into(),
            content: "WTS Infernal Blade 12k".into(),
        },
    ]);

    let last_visible_message = move || {
        let selected = selected_channels.get();
        messages
            .get()
            .into_iter()
            .rev()
            .find(|m| selected.contains(&m.channel))
    };

    let filtered_messages = move || {
        let selected = selected_channels.get();
        messages
            .get()
            .into_iter()
            .filter(|m| selected.contains(&m.channel))
            .collect::<Vec<_>>()
    };

    let toggle_channel = move |channel: ChatChannel| {
        selected_channels.update(|set| {
            if set.contains(&channel) {
                set.remove(&channel);
            } else {
                set.insert(channel);
            }
        });
    };

    let send_message = move || {
        let content = input_value.get();
        if content.trim().is_empty() {
            return;
        }

        let new_id = messages.get().len() as u64 + 1;

        messages.update(|list| {
            list.push(ChatMessage {
                id: new_id,
                channel: ChatChannel::Global,
                author: "You".into(),
                content,
            })
        });

        input_value.set(String::new());
    };

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

                                    <label class="flex items-center gap-1 text-gray-400 hover:text-white cursor-pointer">
                                        <input
                                            type="checkbox"
                                            class="accent-amber-500"
                                            prop:checked=move || {
                                                selected_channels.get().contains(&ChatChannel::Global)
                                            }
                                            on:change=move |_| toggle_channel(ChatChannel::Global)
                                        />
                                        "Global"
                                    </label>

                                    <label class="flex items-center gap-1 text-gray-400 hover:text-white cursor-pointer">
                                        <input
                                            type="checkbox"
                                            class="accent-amber-500"
                                            prop:checked=move || {
                                                selected_channels.get().contains(&ChatChannel::Trade)
                                            }
                                            on:change=move |_| toggle_channel(ChatChannel::Trade)
                                        />
                                        "Trade"
                                    </label>

                                    <label class="flex items-center gap-1 text-gray-400 hover:text-white cursor-pointer">
                                        <input
                                            type="checkbox"
                                            class="accent-amber-500"
                                            prop:checked=move || {
                                                selected_channels.get().contains(&ChatChannel::System)
                                            }
                                            on:change=move |_| toggle_channel(ChatChannel::System)
                                        />
                                        "System"
                                    </label>

                                </div>

                                <div class="flex gap-3 text-gray-400">
                                    <button
                                        class="hover:text-white"
                                        on:click=move |_| minimized.update(|m| *m = !*m)
                                    >
                                        {move || if minimized.get() { "▲" } else { "—" }}
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
                                                    format!("{}: {}", msg.author, msg.content)
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
                                        <div class="flex-1 overflow-y-auto px-4 py-3 space-y-2 bg-zinc-900/70 max-h-[320px]">
                                            <For
                                                each=filtered_messages
                                                key=|msg| msg.id
                                                children=move |msg| {
                                                    view! {
                                                        <div class="text-[13px] leading-snug">
                                                            <span class=move || {
                                                                match msg.channel {
                                                                    ChatChannel::System => "text-amber-400",
                                                                    ChatChannel::Trade => "text-emerald-400",
                                                                    ChatChannel::Global => "text-amber-400",
                                                                }
                                                            }>{msg.author.clone()}</span>
                                                            <span class="text-gray-500">": "</span>
                                                            <span class="text-gray-200">{msg.content.clone()}</span>
                                                        </div>
                                                    }
                                                }
                                            />
                                        </div>

                                        // Input
                                        // <div class="border-t border-zinc-700 bg-zinc-800/70 px-3 py-2">
                                        <div class="border-t border-zinc-700 bg-zinc-900/80 ">
                                            <textarea
                                                class="w-full resize-none px-3 py-2 text-gray-200 focus:outline-none focus:ring-1 focus:ring-amber-500 focus:border-amber-500"
                                                rows="2"
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
