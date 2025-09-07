use leptos::{html::*, prelude::*};

#[component]
pub fn MenuButton(
    #[prop(optional)] disabled: Option<Signal<bool>>,
    #[prop(optional)] button_type: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            class="btn
            text-white font-bold text-shadow shadow-neutral-950
            py-1 lg:py-2 px-2 lg:px-4 rounded shadow-md
            text-sm lg:text-base 
            border border-neutral-950
            bg-gradient-to-t from-zinc-900 to-zinc-800 
            overflow-hidden
            hover:bg-gradient-to-tr hover:from-zinc-900 hover:to-neutral-700 
            active:bg-gradient-to-t active:from-zinc-900 active:to-zinc-950 
            w-auto
            disabled:from-zinc-700 disabled:to-zinc-600
            disabled:text-zinc-400
            disabled:opacity-60 disabled:shadow-none
            "
            type=button_type
            disabled=disabled
        >
            {children()}
        </button>
    }
}

#[component]
pub fn MenuButtonRed(
    #[prop(optional)] disabled: Option<Signal<bool>>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            class="btn
            text-red-300 font-bold text-shadow shadow-neutral-950
            py-1 lg:py-2 px-2 lg:px-4 rounded shadow-md
            text-sm lg:text-base 
            border border-red-800
            bg-gradient-to-t from-red-900 to-red-800
            overflow-hidden
            hover:bg-gradient-to-tr hover:from-red-800 hover:to-red-700
            active:bg-gradient-to-t active:from-red-900 active:to-red-950
            disabled:from-zinc-700 disabled:to-zinc-600
            disabled:text-zinc-400
            disabled:opacity-60 disabled:shadow-none
            "
            disabled=disabled
        >
            {children()}
        </button>
    }
}

#[component]
pub fn FancyButton(
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            class="btn
            text-white font-bold text-shadow shadow-neutral-950
            px-2 lg:px-3 rounded shadow-md
            text-sm lg:text-base 
            border border-neutral-950
            bg-gradient-to-t from-zinc-900 to-zinc-800 
            overflow-hidden
            hover:bg-gradient-to-tr hover:from-zinc-900 hover:to-neutral-700 
            active:bg-gradient-to-t active:from-zinc-900 active:to-zinc-950 
            disabled:from-zinc-700 disabled:to-zinc-600
            disabled:text-zinc-400
            disabled:opacity-60 disabled:shadow-none
            "
            disabled=disabled
        >
            // disabled=disabled
            {children()}
        </button>
    }
}

#[component]
pub fn Toggle(
    #[prop(default = false)] initial: bool,
    mut toggle_callback: impl FnMut(bool) + 'static,
    children: Children,
) -> impl IntoView {
    let checked: RwSignal<bool> = RwSignal::new(initial);
    let switch_value = move |_| {
        let new_value = !checked.get();
        checked.set(new_value);
        toggle_callback(new_value);
    };

    let toggle_class = move || {
        if checked.get() {
            "shadow-md text-white"
            // "ring-2 ring-amber-600/20 shadow-md text-white "
        } else {
            "opacity-60 shadow-none text-zinc-400"
        }
    };

    view! {
        <button
            on:click=switch_value
            class=move || {
                format!(
                    "btn
                    px-2 lg:px-3
                    text-sm lg:text-base 
                    font-bold text-shadow shadow-neutral-950
                    border border-neutral-950 rounded 
                    bg-gradient-to-t from-zinc-900 to-zinc-800 
                    hover:bg-gradient-to-tr hover:from-zinc-900 hover:to-neutral-700
                    active:bg-gradient-to-t active:from-zinc-900 active:to-zinc-950
                    transition-all duration-200
                    relative
                    group
                    {}
                    ",
                    toggle_class(),
                )
            }
        >
            {children()}
        </button>
    }
}

#[component]
pub fn TabButton(children: Children, #[prop(into)] is_active: Signal<bool>) -> impl IntoView {
    let active_class = |active| {
        if active {
            "bg-gradient-to-t from-zinc-900 to-zinc-950"
        } else {
            "
            cursor-pointer
            bg-gradient-to-t from-zinc-900 to-zinc-800 
            hover:bg-gradient-to-tr hover:from-zinc-900 hover:to-neutral-700
            active:bg-gradient-to-t active:from-zinc-900 active:to-zinc-950
            "
        }
    };

    view! {
        <button
            class=move || {
                format!(
                    "btn
                    flex-1
                    px-1 lg:px-3 py-1 lg:py-2 
                    text-sm lg:text-base 
                    font-bold text-shadow
                    border-t border-l border-r border-neutral-950 rounded-t-md 
                    transition-all duration-200
                    relative
                    group
                    {}
                    ",
                    active_class(is_active.get()),
                )
            }
            disabled=is_active
        >
            {children()}
        </button>
    }
}

#[component]
pub fn CloseButton() -> impl IntoView {
    view! {
        <button class="btn ml-2 text-white hover:text-gray-400 transition-colors">
            <svg
                xmlns="http://www.w3.org/2000/svg"
                width="16"
                height="16"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2"
                stroke-linecap="round"
                stroke-linejoin="round"
            >
                <line x1="18" y1="6" x2="6" y2="18" />
                <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
        </button>
    }
}
