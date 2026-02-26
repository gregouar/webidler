use std::collections::HashSet;

use leptos::ev::mousemove;
use leptos::ev::mouseup;
use leptos::prelude::*;

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
enum Channel {
    Global,
    Trade,
    System,
}

#[derive(Clone)]
struct ChatMessage {
    id: u64,
    channel: Channel,
    author: String,
    content: String,
}

#[component]
pub fn ChatPanel(open: RwSignal<bool>) -> impl IntoView {
    let minimized = RwSignal::new(false);

    // Drag state
    let position = RwSignal::new((40i32, 40i32)); // bottom, right offsets
    let dragging = RwSignal::new(false);
    let drag_origin = RwSignal::new((0i32, 0i32));
    let start_drag = move |ev: leptos::ev::MouseEvent| {
        dragging.set(true);
        drag_origin.set((ev.client_x(), ev.client_y()));

        let move_listener = window_event_listener(mousemove, move |ev| {
            if dragging.get() {
                let (ox, oy) = drag_origin.get();
                let dx = ev.client_x() - ox;
                let dy = ev.client_y() - oy;

                position.update(|(b, r)| {
                    let win = window();
                    let height = win.inner_height().unwrap().as_f64().unwrap() as i32;
                    let width = win.inner_width().unwrap().as_f64().unwrap() as i32;

                    *b = (*b - dy).clamp(0, height - 100);
                    *r = (*r - dx).clamp(0, width - 200);
                });

                drag_origin.set((ev.client_x(), ev.client_y()));
            }
        });

        let up_listener = window_event_listener(mouseup, move |_| {
            dragging.set(false);
        });
        drop(move_listener);
        drop(up_listener);
    };

    let selected_channels = RwSignal::new({
        let mut set = HashSet::new();
        set.insert(Channel::Global);
        set.insert(Channel::System);
        set
    });

    let input_value = RwSignal::new(String::new());

    let messages = RwSignal::new(vec![
        ChatMessage {
            id: 1,
            channel: Channel::System,
            author: "System".into(),
            content: "World event starting in 2 minutes.".into(),
        },
        ChatMessage {
            id: 2,
            channel: Channel::Global,
            author: "Nyx".into(),
            content: "Anyone pushing wave 200?".into(),
        },
        ChatMessage {
            id: 3,
            channel: Channel::Trade,
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

    let toggle_channel = move |channel: Channel| {
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
                channel: Channel::Global,
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
                let (bottom, right) = position.get();

                view! {
                    <div
                        class="fixed z-50 select-none text-left"
                        style=format!("bottom:{}px; right:{}px;", bottom, right)
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
                                                selected_channels.get().contains(&Channel::Global)
                                            }
                                            on:change=move |_| toggle_channel(Channel::Global)
                                        />
                                        "Global"
                                    </label>

                                    <label class="flex items-center gap-1 text-gray-400 hover:text-white cursor-pointer">
                                        <input
                                            type="checkbox"
                                            class="accent-amber-500"
                                            prop:checked=move || {
                                                selected_channels.get().contains(&Channel::Trade)
                                            }
                                            on:change=move |_| toggle_channel(Channel::Trade)
                                        />
                                        "Trade"
                                    </label>

                                    <label class="flex items-center gap-1 text-gray-400 hover:text-white cursor-pointer">
                                        <input
                                            type="checkbox"
                                            class="accent-amber-500"
                                            prop:checked=move || {
                                                selected_channels.get().contains(&Channel::System)
                                            }
                                            on:change=move |_| toggle_channel(Channel::System)
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
                                                                    Channel::System => "text-amber-400",
                                                                    Channel::Trade => "text-emerald-400",
                                                                    Channel::Global => "text-amber-400",
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
