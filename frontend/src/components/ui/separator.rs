use leptos::prelude::*;

#[component]

pub fn Separator() -> impl IntoView {
    // <hr class="border-t border-gray-700" />
    view! { <div class="h-px bg-gradient-to-r from-transparent via-zinc-600 to-transparent my-1" /> }
}
