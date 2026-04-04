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
        <div class="relative w-60 text-sm xl:text-base text-white" node_ref=node_ref>
            <button
                on:click=toggle
                class=move || {
                    format!(
                        "btn relative isolate overflow-hidden
                        w-full flex items-center justify-between gap-2
                        px-1 xl:px-3 py-1 xl:py-2 rounded-[4px] xl:rounded-[6px]
                        tracking-[0.08em] text-stone-100 font-extrabold text-shadow shadow-black/90
                        border border-[#6c5329]
                        shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(230,208,154,0.22),inset_0_-1px_0_rgba(0,0,0,0.45)]
                        before:pointer-events-none before:absolute before:inset-[1px]
                        before:rounded-[3px] xl:before:rounded-[5px]
                        before:border before:border-[#d5b16d]/18
                        before:bg-[linear-gradient(180deg,rgba(222,188,112,0.08),transparent_36%)]
                        focus:outline-none {}",
                        if is_open.get() {
                            "border-[#9c7841] before:opacity-0 brightness-90 text-zinc-100
                            shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(26,17,10,0.95),inset_0_3px_5px_rgba(0,0,0,0.55),inset_0_-1px_0_rgba(0,0,0,0.22)]"
                        } else {
                            "hover:border-[#a27f46] hover:text-[#f3ead2]
                            hover:shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(244,225,181,0.28),inset_0_-1px_0_rgba(0,0,0,0.45)]"
                        },
                    )
                }
                style="
                background-image:
                linear-gradient(180deg, rgba(214,177,102,0.10), rgba(0,0,0,0.18)),
                linear-gradient(180deg, rgba(43,40,46,0.96), rgba(20,19,23,1));
                background-size: auto, auto, 180px 180px;
                background-position: center, center, center;
                background-blend-mode: screen, normal, soft-light;
                "
            >
                <span class="pointer-events-none absolute inset-x-2 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/55 to-transparent"></span>
                <span class="pointer-events-none absolute left-[2px] top-[2px] bottom-[2px] w-px bg-gradient-to-b from-[#f0d79f]/35 via-transparent to-black/40"></span>
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
                    "dropdown-transition absolute mt-1 w-full rounded-[6px]
                    border border-[#6c5329]
                    shadow-[0_10px_24px_rgba(0,0,0,0.42),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(230,208,154,0.12)]
                    max-h-80 overflow-auto z-20 {}",
                    if is_open.get() { "open" } else { "" },
                )
            }
                style="
                background-image:
                linear-gradient(180deg, rgba(214,177,102,0.08), rgba(0,0,0,0.16)),
                linear-gradient(180deg, rgba(35,33,39,0.98), rgba(17,16,20,1));
                background-size: auto, auto, 180px 180px;
                background-position: center, center, center;
                background-blend-mode: screen, normal, soft-light;
                "
            >
                {options
                    .into_iter()
                    .map(|(opt, text)| {
                        view! {
                            <li
                                on:click=move |_| select_option(opt.clone())
                                class="cursor-pointer px-4 py-2 text-zinc-200 border-b border-black/20
                                hover:bg-[#3a3430] hover:text-[#f1e4c4] last:border-b-0"
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
        <div class="relative w-60 text-sm xl:text-base text-white" node_ref=node_ref>
            <button
                on:click=toggle
                class=move || {
                    format!(
                        "btn relative isolate overflow-hidden
                        w-full flex items-center justify-between gap-2
                        px-1 xl:px-3 py-1 xl:py-2 rounded-[4px] xl:rounded-[6px]
                        tracking-[0.08em] text-stone-100 font-extrabold text-shadow shadow-black/90
                        border border-[#6c5329]
                        shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(230,208,154,0.22),inset_0_-1px_0_rgba(0,0,0,0.45)]
                        before:pointer-events-none before:absolute before:inset-[1px]
                        before:rounded-[3px] xl:before:rounded-[5px]
                        before:border before:border-[#d5b16d]/18
                        before:bg-[linear-gradient(180deg,rgba(222,188,112,0.08),transparent_36%)]
                        focus:outline-none {}",
                        if is_open.get() {
                            "border-[#9c7841] before:opacity-0 brightness-90 text-zinc-100
                            shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(26,17,10,0.95),inset_0_3px_5px_rgba(0,0,0,0.55),inset_0_-1px_0_rgba(0,0,0,0.22)]"
                        } else {
                            "hover:border-[#a27f46] hover:text-[#f3ead2]
                            hover:shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(244,225,181,0.28),inset_0_-1px_0_rgba(0,0,0,0.45)]"
                        },
                    )
                }
                style="
                background-image:
                linear-gradient(180deg, rgba(214,177,102,0.10), rgba(0,0,0,0.18)),
                linear-gradient(180deg, rgba(43,40,46,0.96), rgba(20,19,23,1));
                background-size: auto, auto, 180px 180px;
                background-position: center, center, center;
                background-blend-mode: screen, normal, soft-light;
                "
            >
                <span class="pointer-events-none absolute inset-x-2 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/55 to-transparent"></span>
                <span class="pointer-events-none absolute left-[2px] top-[2px] bottom-[2px] w-px bg-gradient-to-b from-[#f0d79f]/35 via-transparent to-black/40"></span>
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
                    "dropdown-transition absolute mt-1 w-full rounded-[6px]
                    border border-[#6c5329]
                    shadow-[0_10px_24px_rgba(0,0,0,0.42),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(230,208,154,0.12)]
                    z-20 {}",
                    if is_open.get() { "open" } else { "" },
                )
            }
                style="
                background-image:
                linear-gradient(180deg, rgba(214,177,102,0.08), rgba(0,0,0,0.16)),
                linear-gradient(180deg, rgba(35,33,39,0.98), rgba(17,16,20,1));
                background-size: auto, auto, 180px 180px;
                background-position: center, center, center;
                background-blend-mode: screen, normal, soft-light;
                "
            >
                <div class="px-1 xl:px-3 py-1 xl:py-2 border-b border-black/25 bg-black/10">
                    <input
                        node_ref=search_ref
                        class="w-full bg-transparent text-zinc-100 placeholder:text-zinc-500 focus:outline-none"
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
                                        class="cursor-pointer px-4 py-2 text-zinc-200 border-b border-black/20
                                        hover:bg-[#3a3430] hover:text-[#f1e4c4] last:border-b-0"
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
                            .then(|| view! { <li class="px-4 py-2 text-zinc-500">"No elements found."</li> })
                    }}
                </ul>
            </div>
        </div>
    }
}
