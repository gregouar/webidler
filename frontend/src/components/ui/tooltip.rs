use leptos::html::*;
use leptos::prelude::*;
use leptos_use::use_mouse;

// TODO: position
#[component]
pub fn Tooltip(#[prop(into)] show: Signal<bool>, children: ChildrenFn) -> impl IntoView {
    let mouse = use_mouse();

    // See: https://book.leptos.dev/interlude_projecting_children.html
    let children = StoredValue::new(children);

    view! {
        <Show when=move || {
            show.get()
        }>
            {move || {
                view! {
                    <div
                        class="fixed z-50 pointer-events-none transition-opacity duration-150"
                        style=format!(
                            "top: {}px; left: {}px;",
                            mouse.y.get() + 16.0,
                            mouse.x.get() + 16.0,
                        )
                    >
                        {children.read_value()()}
                    </div>
                }
            }}
        </Show>
    }
}
