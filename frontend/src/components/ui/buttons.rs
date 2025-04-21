use leptos::html::*;
use leptos::prelude::*;

#[component]
pub fn MenuButton(
    // #[prop(optional)] disabled: Option<bool>,
    children: Children,
) -> impl IntoView {
    view! {
        <button class="
        text-white font-bold text-shadow shadow-neutral-950
        py-2 px-4 rounded shadow-md
        border border-neutral-950
        bg-gradient-to-t from-zinc-900 to-zinc-800 
        overflow-hidden
        hover:bg-gradient-to-tr hover:from-zinc-900 hover:to-neutral-700 
        active:bg-gradient-to-t active:from-zinc-900 active:to-zinc-950 
        ">
            // disabled=disabled
            {children()}
        </button>
    }
}

#[component]
pub fn Toggle(
    #[prop(default = "".to_string())] label: String,
    #[prop(default = false)] initial: bool,
    mut toggle_callback: impl FnMut(bool) + 'static,
) -> impl IntoView {
    let checked: RwSignal<bool> = RwSignal::new(initial);
    let switch_value = move |_| {
        let new_value = !checked.get();
        checked.set(new_value);
        toggle_callback(new_value);
    };
    view! {
        <div class="flex items-center space-x-3">
            <span class="text-white font-semibold">{label}</span>
            <label class="relative inline-flex items-center cursor-pointer group">
                <input type="checkbox" class="sr-only peer" checked=initial on:input=switch_value />
                <div class="
                w-12 h-6 
                bg-gradient-to-t from-zinc-900 to-zinc-800 
                rounded-full 
                border border-neutral-950 
                shadow-md 
                peer-checked:bg-gradient-to-t peer-checked:from-amber-800 peer-checked:to-amber-600 
                transition-colors duration-300
                "></div>
                <div class="
                absolute left-0.5 top-0.5 
                w-5 h-5 
                bg-zinc-300 
                rounded-full 
                shadow 
                transition-transform duration-300 
                peer-checked:translate-x-6 
                peer-checked:bg-white
                "></div>
            </label>
        </div>
    }
}
