use leptos::{html::*, prelude::*};

#[component]
pub fn Checkbox(
    label: &'static str,
    mut on_change: impl FnMut(bool) + 'static,
    #[prop(into)] checked: Signal<bool>,
) -> impl IntoView {
    view! {
        <label class="flex items-center gap-1 text-gray-400 hover:text-white cursor-pointer">
            <input
                type="checkbox"
                class="appearance-none w-4 h-4 rounded-xs
                border border-zinc-500 bg-zinc-700 
                checked:bg-amber-500 checked:border-amber-500 
                checked:[&:after]:content-['✓']
                checked:[&:after]:text-zinc-950
                checked:[&:after]:font-bold
                checked:[&:after]:text-[12px]
                checked:[&:after]:flex
                checked:[&:after]:items-center
                checked:[&:after]:justify-center
                flex items-center justify-center
                active:bg-amber-600
                hover:outline-none hover:ring-1 hover:ring-amber-500"
                prop:checked=checked
                on:change=move |ev| {
                    on_change(event_target_checked(&ev));
                }
            />
            {label}
        </label>
    }
}
