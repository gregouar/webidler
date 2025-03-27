use leptos::html::*;
use leptos::prelude::*;

#[component]
pub fn MainMenuButton(children: Children) -> impl IntoView {
    view! {
        <button
            class="
                text-white font-bold text-shadow shadow-neutral-950
                py-2 px-4 rounded shadow-md
                border border-neutral-950
                bg-gradient-to-t from-zinc-900 to-zinc-800 
                overflow-hidden
                hover:bg-gradient-to-tr hover:from-zinc-900 hover:to-neutral-700 
                active:bg-gradient-to-t active:from-zinc-900 active:to-zinc-950 
            ">
            {children()}
        </button>
    }
}
