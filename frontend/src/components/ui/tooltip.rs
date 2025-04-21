use leptos::ev;
use leptos::html::*;
use leptos::prelude::*;

// TODO: position
#[component]
pub fn Tooltip(#[prop(into)] show: Signal<bool>, children: ChildrenFn) -> impl IntoView {
    let (mouse_pos, set_mouse_pos) = signal((0, 0));

    // Global mouse listener
    let _ = window_event_listener(ev::mousemove, move |ev| {
        set_mouse_pos.set((ev.client_x(), ev.client_y()));
    });

    // See: https://book.leptos.dev/interlude_projecting_children.html
    let children = StoredValue::new(children);

    view! {
        <Show when=move || {
            show.get()
        }>
            {move || {
                let (x, y) = mouse_pos.get();
                view! {
                    <div
                        class="fixed z-50 pointer-events-none bg-black/90 text-white text-sm p-2 rounded shadow-xl transition-opacity duration-150 ring-1"
                        style=format!("top: {}px; left: {}px;", y + 16, x + 16)
                    >
                        {children.read_value()()}
                    </div>
                }
            }}
        </Show>
    }
}
