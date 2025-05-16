use std::time::Duration;

use leptos::ev::KeyboardEvent;
use leptos::html::*;
use leptos::prelude::*;

#[component]
pub fn MenuPanel(open: RwSignal<bool>, children: ChildrenFn) -> impl IntoView {
    let panel_ref = NodeRef::<Div>::new();

    Effect::new(move |_| {
        if open.get() {
            if let Some(el) = panel_ref.get_untracked() {
                _ = el.focus(); // Set focus on the panel
            }
        }
    });

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

    // See: https://book.leptos.dev/interlude_projecting_children.html
    let children = StoredValue::new(children);

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
                class="absolute h-full inset-0 bg-black/70 z-40"
                on:click=move |_| open.set(false)
                on:keydown=handle_key
                // allow it to receive keyboard events
                tabindex="0"
            >
                <div
                    class="w-full z-41"
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
                    {children.read_value()()}
                </div>
            </div>
        </Show>
    }
}
