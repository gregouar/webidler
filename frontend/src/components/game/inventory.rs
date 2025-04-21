use std::time::Duration;

use leptos::ev::KeyboardEvent;
use leptos::html::*;
use leptos::prelude::*;

use super::player_card::PlayerCard;

#[component]
pub fn Inventory(open: RwSignal<bool>) -> impl IntoView {
    let panel_ref = NodeRef::<Div>::new();

    // Focus the panel when it opens
    Effect::new(move |_| {
        if open.get() {
            if let Some(el) = panel_ref.get_untracked() {
                _ = el.focus(); // Set focus on the panel
            }
        }
    });

    // Handle Escape key
    let handle_key = move |e: KeyboardEvent| {
        if e.key() == "Escape" {
            open.set(false);
        }
    };

    let (is_visible, set_is_visible) = signal(false);
    Effect::new(move |_| {
        if open.get() {
            set_is_visible.set(true);
        } else {
            set_timeout(
                move || set_is_visible.set(open.get()),
                Duration::from_millis(300),
            );
        }
    });

    view! {
        <style>
            "@keyframes dropDown {
                from {
                    transform: translateY(-100%);
                }
                to {
                    transform: translateY(0);
                }
            }" "@keyframes pullUp {
            from {
            transform: translateY(0);
            }
            to {
            transform: translateY(-100%);
            }
            }"
        </style>
        <Show when=move || is_visible.get()>
            <div
                class="absolute h-full inset-0 bg-black bg-opacity-50 z-40"
                on:click=move |_| open.set(false)
                on:keydown=handle_key
                // allow it to receive keyboard events
                tabindex="0"
            >
                <div
                    class="w-full grid grid-cols-3 justify-items-stretch flex items-start gap-4 p-4 transition-all duration-300 transform translate-y-0"
                    style=move || {
                        if open.get() {
                            "animation: dropDown 0.3s ease-out forwards;"
                        } else {
                            "animation: pullUp 0.3s ease-out forwards;"
                        }
                    }
                    // prevent background click from closing it
                    on:click=|e| e.stop_propagation()
                >
                    <PlayerCard class:col-span-1 class:justify-self-end />
                    <ItemsGrid class:col-span-2 class:justify-self-start />
                </div>
            </div>
        </Show>
    }
}

#[component]
fn ItemsGrid() -> impl IntoView {
    view! {
        <div class="bg-zinc-800 rounded-md h-full w-full shadow-lg ring-1 ring-zinc-950 overflow-hidden">
            "Some items"
        </div>
    }
}
