use leptos::prelude::*;

#[component]

pub fn Separator() -> impl IntoView {
    view! { <div class="h-px bg-gradient-to-r from-transparent via-zinc-600 to-transparent my-1" /> }
}

#[component]

pub fn TitleSeparator() -> impl IntoView {
    view! {
        <div class="h-px bg-gradient-to-r from-transparent via-[#edd39a]/30 to-transparent my-1" />
    }
}
