use indexmap::IndexMap;
use leptos::{prelude::*, web_sys};
use leptos_use::on_click_outside;

#[component]
pub fn DropdownMenu<T>(
    options: IndexMap<T, String>,
    chosen_option: RwSignal<T>,
    #[prop(default = "Select an option")] missing_text: &'static str,
) -> impl IntoView
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
            "
        </style>

        <div class="relative w-60 text-sm xl:text-base text-white" node_ref=node_ref>
            <button
                on:click=toggle
                class=move || {
                    format!(
                        "btn w-full flex items-center justify-between gap-2
                        px-1 xl:px-3 py-1 xl:py-2 rounded-md
                        shadow-md border border-zinc-950 focus:outline-none {}",
                        if is_open.get() {
                            "bg-gradient-to-t from-zinc-900 to-zinc-950 "
                        } else {
                            "bg-gradient-to-t from-zinc-900 to-zinc-800 hover:from-zinc-800 hover:to-zinc-700"
                        },
                    )
                }
            >
                <span class="truncate flex-1 min-w-0">
                    {
                        let options = options.clone();
                        move || {
                            options
                                .get(&chosen_option.get())
                                .cloned()
                                .unwrap_or(missing_text.to_string())
                        }
                    }
                </span>
                <span class=move || {
                    format!(
                        "shrink-0 transition-transform duration-200 {}",
                        if is_open.get() { "rotate-180" } else { "rotate-0" },
                    )
                }>"▼"</span>
            </button>

            <ul class=move || {
                format!(
                    "dropdown-transition absolute mt-1 w-full rounded-md bg-zinc-800 border border-zinc-950
                    shadow-lg max-h-80 overflow-auto z-20 {}",
                    if is_open.get() { "open" } else { "" },
                )
            }>
                {options
                    .into_iter()
                    .map(|(opt, text)| {
                        view! {
                            <li
                                on:click=move |_| select_option(opt.clone())
                                class="cursor-pointer px-4 py-2 hover:bg-zinc-700"
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

#[component]
pub fn SearchableDropdownMenu<T>(
    options: IndexMap<T, String>,
    chosen_option: RwSignal<T>,
    #[prop(default = "Select an option")] missing_text: &'static str,
) -> impl IntoView
where
    T: Clone + std::hash::Hash + Eq + Send + Sync + 'static,
{
    let node_ref = NodeRef::new();
    let search_ref = NodeRef::<leptos::html::Input>::new();

    let is_open = RwSignal::new(false);
    let search = RwSignal::new(String::new());

    let toggle = move |_| {
        is_open.update(|open| {
            *open = !*open;
            if *open && let Some(input) = search_ref.get() {
                input.focus().unwrap();
            }
            //     search.set("".to_string());
        })
    };
    let _ = on_click_outside(node_ref, move |_| is_open.set(false));

    let select_option = move |opt| {
        is_open.set(false);
        chosen_option.set(opt);
    };

    let filtered_options = Signal::derive({
        let options = options.clone();
        move || {
            let term = search.get().to_lowercase();
            options
                .iter()
                .filter(move |(_, text)| term.is_empty() || text.to_lowercase().contains(&term))
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect::<Vec<_>>()
        }
    });

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
            ul::-webkit-scrollbar { width: 8px; }
            ul::-webkit-scrollbar-track { background: #1f1f1f; border-radius: 4px; }
            ul::-webkit-scrollbar-thumb { background-color: #525252; border-radius: 4px; border: 2px solid #1f1f1f; }
            ul { scrollbar-width: thin; scrollbar-color: #525252 #1f1f1f; }
            ul::-webkit-scrollbar-thumb:hover { background-color: #737373; }
            "
        </style>

        <div class="relative w-60 text-sm xl:text-base text-white" node_ref=node_ref>
            <button
                on:click=toggle
                class=move || {
                    format!(
                        "btn w-full flex items-center justify-between gap-2
                        px-1 xl:px-3 py-1 xl:py-2 rounded-md
                        shadow-md border border-zinc-950 focus:outline-none {}",
                        if is_open.get() {
                            "bg-gradient-to-t from-zinc-900 to-zinc-950 "
                        } else {
                            "bg-gradient-to-t from-zinc-900 to-zinc-800 hover:from-zinc-800 hover:to-zinc-700"
                        },
                    )
                }
            >
                <span class="truncate flex-1 min-w-0">
                    {
                        let options = options.clone();
                        move || {
                            options
                                .get(&chosen_option.get())
                                .cloned()
                                .unwrap_or(missing_text.to_string())
                        }
                    }
                </span>
                <span class=move || {
                    format!(
                        "shrink-0 transition-transform duration-200 {}",
                        if is_open.get() { "rotate-180" } else { "rotate-0" },
                    )
                }>"▼"</span>
            </button>

            <div class=move || {
                format!(
                    "dropdown-transition absolute mt-1 w-full rounded-md bg-zinc-800 border border-zinc-950
                    shadow-lg z-20 {}",
                    if is_open.get() { "open" } else { "" },
                )
            }>
                <div class="px-1 xl:px-3 py-1 xl:py-2 border-b border-zinc-700 bg-gray-800">
                    <input
                        node_ref=search_ref
                        class="w-full bg-gray-800 focus:outline-none"
                        placeholder="Search..."
                        prop:value=move || search.get()
                        on:input=move |ev| search.set(event_target_value(&ev))

                        on:keydown=move |ev| {
                            if ev.key() == "Enter" {
                                ev.prevent_default();
                                if let Some(el) = search_ref.get() {
                                    let input: web_sys::HtmlInputElement = el;
                                    let _ = input.blur();
                                }
                            }
                        }
                    />

                </div>

                <ul class="max-h-80 overflow-auto text-left">
                    {move || {
                        filtered_options
                            .get()
                            .into_iter()
                            .map(|(opt, text)| {
                                view! {
                                    <li
                                        on:click=move |_| select_option(opt.clone())
                                        class="cursor-pointer px-4 py-2 hover:bg-zinc-700"
                                    >
                                        {text}
                                    </li>
                                }
                            })
                            .collect::<Vec<_>>()
                    }}
                    {move || {
                        filtered_options
                            .read()
                            .is_empty()
                            .then(|| {
                                view! { <li class="px-4 py-2">"No elements found."</li> }
                            })
                    }}
                </ul>
            </div>
        </div>
    }
}
