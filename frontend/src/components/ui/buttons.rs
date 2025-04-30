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
pub fn FancyButton(disabled: Signal<bool>, children: Children) -> impl IntoView {
    view! {
        <button
            class="
            text-white font-bold text-shadow shadow-neutral-950
            px-3 rounded shadow-md
            border border-neutral-950
            bg-gradient-to-t from-zinc-900 to-zinc-800 
            overflow-hidden
            hover:bg-gradient-to-tr hover:from-zinc-900 hover:to-neutral-700 
            active:bg-gradient-to-t active:from-zinc-900 active:to-zinc-950 
            disabled:from-zinc-700 disabled:to-zinc-600
            disabled:text-zinc-400 disabled:pointer-events-none
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

    let toggle_class = move || {
        if checked.get() {
            "shadow-md text-white"
            // "ring-2 ring-amber-600/20 shadow-md text-white "
        } else {
            "opacity-60 shadow-none"
        }
    };

    view! {
        <button
            on:click=switch_value
            class=move || {
                format!(
                    "
                    px-3 
                    font-bold text-shadow shadow-neutral-950
                    border border-neutral-950 rounded 
                    bg-gradient-to-t from-zinc-900 to-zinc-800 
                    hover:bg-gradient-to-tr hover:from-zinc-900 hover:to-neutral-700
                    active:bg-gradient-to-t active:from-zinc-900 active:to-zinc-950
                    transition-all duration-200
                    relative
                    group
                    [font-variant:small-caps]
                    {}
                    ",
                    toggle_class(),
                )
            }
        >
            {label}
        </button>
    }
}

// #[component]
// pub fn Toggle(
//     #[prop(default = "".to_string())] label: String,
//     #[prop(default = false)] initial: bool,
//     mut toggle_callback: impl FnMut(bool) + 'static,
// ) -> impl IntoView {
//     let checked: RwSignal<bool> = RwSignal::new(initial);
//     let switch_value = move |_| {
//         let new_value = !checked.get();
//         checked.set(new_value);
//         toggle_callback(new_value);
//     };
//     view! {
//         <div class="flex items-center space-x-3">
//             <span class="text-white font-semibold">{label}</span>
//             <label class="relative inline-flex items-center cursor-pointer group">
//                 <input type="checkbox" class="sr-only peer" checked=initial on:input=switch_value />

//                 <div class="
//                 w-6 h-3 sm:w-8 sm:h-4 md:w-10 md:h-5 lg:w-12 lg:h-6 xl:w-14 xl:h-7
//                 bg-gradient-to-t from-zinc-900 to-zinc-800
//                 rounded-full
//                 border border-neutral-950
//                 shadow-md
//                 peer-checked:bg-gradient-to-t peer-checked:from-amber-800 peer-checked:to-amber-600
//                 transition-colors duration-300
//                 "></div>

//                 <div class="
//                 absolute left-0.5 top-0.5
//                 w-2 h-2 sm:w-3 sm:h-3  md:w-4 md:h-4  lg:w-5 lg:h-5 xl:w-6 xl:h-6
//                 bg-zinc-300
//                 rounded-full
//                 shadow
//                 transition-transform duration-300
//                 peer-checked:translate-x-3 sm:peer-checked:translate-x-4 md:peer-checked:translate-x-5 lg:peer-checked:translate-x-6 xl:peer-checked:translate-x-7
//                 peer-checked:bg-white
//                 "></div>
//             </label>
//         </div>
//     }
// }
