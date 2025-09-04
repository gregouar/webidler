use indexmap::IndexMap;
use leptos::prelude::*;
use leptos_use::on_click_outside;

#[component]
pub fn DropdownMenu<T>(options: IndexMap<T, String>, chosen_option: RwSignal<T>) -> impl IntoView
where
    T: Clone + std::hash::Hash + Eq + Send + Sync + 'static,
{
    let node_ref = NodeRef::new();
    let is_open = RwSignal::new(false);

    let toggle = move |_| is_open.update(|open| *open = !*open);
    let _ = on_click_outside(node_ref, move |_| is_open.set(false));
    let select_option = move |opt| {
        is_open.set(false);
        chosen_option.set(opt);
    };

    view! {
        <style>
            ".dropdown-transition {
            opacity: 0;
            transform: scaleY(0.5);
            transform-origin: top;
            transition: all 150ms ease-out;
            pointer-events: none;
            }
            
            .dropdown-transition.open {
            opacity: 1;
            transform: scaleY(1);
            pointer-events: auto;
            }
            
            ul::-webkit-scrollbar {
            width: 8px;
            }
            
            ul::-webkit-scrollbar-track {
            background: #1f1f1f;
            border-radius: 4px;
            }
            
            ul::-webkit-scrollbar-thumb {
            background-color: #525252;
            border-radius: 4px;
            border: 2px solid #1f1f1f;
            }
            
            ul {
            scrollbar-width: thin;
            scrollbar-color: #525252 #1f1f1f;
            }
            
            ul::-webkit-scrollbar-thumb:hover {
            background-color: #737373;
            }
            "
        </style>

        <div class="relative w-60" node_ref=node_ref>
            <button
                on:click=toggle
                class="w-full text-left px-1 sm:px-2 md:px-3 px-4 py-1 md:py-2 rounded-md
                text-white bg-gradient-to-t from-zinc-900 to-zinc-800 shadow-md border border-zinc-950 
                hover:from-zinc-800 hover:to-zinc-700 focus:outline-none"
            >
                {
                    let options = options.clone();
                    move || {
                        options
                            .get(&chosen_option.get())
                            .cloned()
                            .unwrap_or("Select an option".to_string())
                    }
                }
                <span class="float-right">"▼"</span>
            </button>

            <ul class=move || {
                format!(
                    "dropdown-transition absolute mt-1 w-full rounded-md bg-zinc-800 border border-zinc-950 shadow-lg max-h-80 overflow-auto {}  z-20",
                    if is_open.get() { "open" } else { "" },
                )
            }>
                {options
                    .into_iter()
                    .map(|(opt, text)| {
                        view! {
                            <li
                                on:click=move |_| select_option(opt.clone())
                                class="cursor-pointer px-4 py-2 hover:bg-zinc-700 text-white"
                            >
                                {text}
                            </li>
                        }
                    })
                    .collect::<Vec<_>>()}
            </ul>
        </div>
    }
}
