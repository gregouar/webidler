use leptos::{html::*, prelude::*};

use crate::components::settings::{GraphicsQuality, SettingsContext};

#[component]
pub fn MenuButton(
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional)] button_type: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let settings: SettingsContext = expect_context();

    view! {
        <button
            class=move || {
                let quality_class = match settings.graphics_quality() {
                    GraphicsQuality::High => {
                        "btn relative isolate overflow-hidden
                    tracking-[0.08em]
                    text-stone-100 font-extrabold text-shadow-lg/50 shadow-black/90
                    py-1 xl:py-2 px-2 xl:px-4 rounded-[4px] xl:rounded-[6px]
                    text-sm xl:text-base
                    border border-[#6c5329]
                    shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(230,208,154,0.22),inset_0_-1px_0_rgba(0,0,0,0.45)]
                    before:pointer-events-none before:absolute before:inset-[1px]
                    before:rounded-[3px] xl:before:rounded-[5px]
                    before:border before:border-[#d5b16d]/18
                    before:bg-[linear-gradient(180deg,rgba(222,188,112,0.08),transparent_36%)]
                    hover:border-[#a27f46]
                    hover:text-[#f3ead2]
                    hover:shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(244,225,181,0.28),inset_0_-1px_0_rgba(0,0,0,0.45)]
                    active:translate-y-[1px]
                    active:before:opacity-0 active:brightness-90
                    active:text-white
                    active:shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(26,17,10,0.95),inset_0_3px_5px_rgba(0,0,0,0.55),inset_0_-1px_0_rgba(0,0,0,0.22)]
                    w-auto
                    disabled:text-zinc-500
                    disabled:border-[#4b4030]
                    disabled:opacity-60 disabled:shadow-none
                    disabled:before:hidden"
                    }
                    GraphicsQuality::Medium => {
                        "btn relative isolate overflow-hidden
                    tracking-[0.08em]
                    text-stone-100 font-extrabold text-shadow-lg/50 shadow-black/90
                    py-1 xl:py-2 px-2 xl:px-4 rounded-[4px] xl:rounded-[6px]
                    text-sm xl:text-base
                    border border-[#6c5329]
                    shadow-md
                    before:pointer-events-none before:absolute before:inset-[1px]
                    before:rounded-[3px] xl:before:rounded-[5px]
                    before:border before:border-[#d5b16d]/10
                    hover:border-[#a27f46]
                    hover:text-[#f3ead2]
                    active:translate-y-[1px]
                    active:brightness-90
                    active:text-white
                    w-auto
                    disabled:text-zinc-500
                    disabled:border-[#4b4030]
                    disabled:opacity-60
                     disabled:shadow-none"
                    }
                    GraphicsQuality::Low => {
                        "btn relative isolate overflow-hidden
                    tracking-[0.08em]
                    text-stone-100 font-extrabold
                    py-1 xl:py-2 px-2 xl:px-4 rounded-[4px] xl:rounded-[6px]
                    text-sm xl:text-base
                    border border-[#5f5035]
                    before:pointer-events-none before:absolute before:inset-[1px]
                    before:rounded-[3px] xl:before:rounded-[5px]
                    before:border before:border-[#a88a53]/10
                    hover:border-[#8b7347] hover:text-[#efe1bf]
                    active:translate-y-[1px] active:brightness-95
                    w-auto
                    disabled:text-zinc-500 disabled:border-[#4b4030] disabled:opacity-60"
                    }
                };
                format!("{quality_class} {}", class.unwrap_or_default())
            }
            style:background-image=move || {
                match settings.graphics_quality() {
                    GraphicsQuality::High => {
                        if disabled.map(|d| d.get()).unwrap_or(false) {
                            "linear-gradient(180deg, rgba(110,104,96,0.08), rgba(0,0,0,0.12)), linear-gradient(180deg, rgba(58,55,60,0.92), rgba(34,33,37,1))"
                                .to_string()
                        } else {
                            "linear-gradient(180deg, rgba(214,177,102,0.10), rgba(0,0,0,0.18)), linear-gradient(180deg, rgba(43,40,46,0.96), rgba(20,19,23,1))"
                                .to_string()
                        }
                    }
                    GraphicsQuality::Medium => {
                        if disabled.map(|d| d.get()).unwrap_or(false) {
                            "linear-gradient(180deg, rgba(88,83,78,0.18), rgba(28,28,32,0.94))"
                                .to_string()
                        } else {
                            "linear-gradient(180deg, rgba(170,140,84,0.09), rgba(0,0,0,0.08)), linear-gradient(180deg, rgba(45,42,48,0.98), rgba(23,22,26,1))"
                                .to_string()
                        }
                    }
                    GraphicsQuality::Low => {
                        if disabled.map(|d| d.get()).unwrap_or(false) {
                            "linear-gradient(180deg, rgba(130,113,77,0.04), rgba(0,0,0,0.08)), linear-gradient(180deg, rgba(62,59,63,0.95), rgba(39,38,42,1))"
                                .to_string()
                        } else {
                            "linear-gradient(180deg, rgba(176,145,86,0.05), rgba(0,0,0,0.08)), linear-gradient(180deg, rgba(56,53,58,0.97), rgba(31,30,34,1))"
                                .to_string()
                        }
                    }
                }
            }
            type=button_type
            disabled=disabled
        >
            <Show when=move || settings.graphics_quality() != GraphicsQuality::Low>
                <span class="pointer-events-none absolute inset-x-2 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/55 to-transparent"></span>
            </Show>
            <Show when=move || settings.uses_heavy_effects()>
                <span class="pointer-events-none absolute left-[2px] top-[2px] bottom-[2px] w-px bg-gradient-to-b from-[#f0d79f]/35 via-transparent to-black/40"></span>
            </Show>
            <span class="relative z-10">{children()}</span>
        </button>
    }
}

#[component]
pub fn MenuButtonRed(
    #[prop(optional)] disabled: Option<Signal<bool>>,
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let settings: SettingsContext = expect_context();

    view! {
        <button
            class=move || {
                let quality_class = match settings.graphics_quality() {
                    GraphicsQuality::High => {
                        "btn relative isolate overflow-hidden
                    tracking-[0.08em]
                    text-[#f2c4bb] font-extrabold text-shadow shadow-black/90
                    py-1 xl:py-2 px-2 xl:px-4 rounded-[4px] xl:rounded-[6px]
                    text-sm xl:text-base
                    border border-[#8e4538]
                    shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(34,10,10,0.95),inset_0_1px_0_rgba(255,210,184,0.18),inset_0_-1px_0_rgba(0,0,0,0.45)]
                    before:pointer-events-none before:absolute before:inset-[1px]
                    before:rounded-[3px] xl:before:rounded-[5px]
                    before:border before:border-[#d78b78]/16
                    before:bg-[linear-gradient(180deg,rgba(239,170,142,0.08),transparent_36%)]
                    hover:border-[#b55d4c]
                    hover:text-[#ffd8d0]
                    hover:shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(34,10,10,0.95),inset_0_1px_0_rgba(255,220,198,0.24),inset_0_-1px_0_rgba(0,0,0,0.45)]
                    active:translate-y-[1px]
                    active:before:opacity-0 active:brightness-90
                    active:text-[#d7aca2]
                    active:shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(34,10,10,0.95),inset_0_3px_5px_rgba(0,0,0,0.55),inset_0_-1px_0_rgba(0,0,0,0.22)]
                    disabled:text-zinc-500
                    disabled:border-[#4f312d]
                    disabled:opacity-60 disabled:shadow-none
                    disabled:before:hidden"
                    }
                    GraphicsQuality::Medium => {
                        "btn relative isolate overflow-hidden
                    tracking-[0.08em]
                    text-[#f2c4bb] font-extrabold text-shadow shadow-black/90
                    py-1 xl:py-2 px-2 xl:px-4 rounded-[4px] xl:rounded-[6px]
                    text-sm xl:text-base
                    shadow-md
                    border border-[#8e4538]
                    before:pointer-events-none before:absolute before:inset-[1px]
                    before:rounded-[3px] xl:before:rounded-[5px]
                    before:border before:border-[#d78b78]/10
                    hover:border-[#b55d4c] hover:text-[#ffd8d0]
                    active:translate-y-[1px] active:brightness-90 active:text-[#d7aca2]
                    disabled:text-zinc-500 disabled:border-[#4f312d] disabled:opacity-60
                    disabled:shadow-none"
                    }
                    GraphicsQuality::Low => {
                        "btn relative isolate overflow-hidden
                    tracking-[0.08em]
                    text-[#f2c4bb] font-extrabold
                    py-1 xl:py-2 px-2 xl:px-4 rounded-[4px] xl:rounded-[6px]
                    text-sm xl:text-base
                    border border-[#8e4538]
                    before:pointer-events-none before:absolute before:inset-[1px]
                    before:rounded-[3px] xl:before:rounded-[5px]
                    before:border before:border-[#c08a78]/10
                    hover:border-[#b55d4c] hover:text-[#ffd8d0]
                    active:translate-y-[1px] active:brightness-95
                    disabled:text-zinc-500 disabled:border-[#4f312d] disabled:opacity-60"
                    }
                };
                format!("{quality_class} {}", class.unwrap_or_default())
            }
            style:background-image=move || {
                match settings.graphics_quality() {
                    GraphicsQuality::High => {
                        "linear-gradient(180deg, rgba(230,164,125,0.14), rgba(0,0,0,0.18)), linear-gradient(180deg, rgba(72,28,26,0.98), rgba(35,11,13,1))"
                            .to_string()
                    }
                    GraphicsQuality::Medium => {
                        "linear-gradient(180deg, rgba(209,126,96,0.10), rgba(0,0,0,0.08)), linear-gradient(180deg, rgba(78,32,31,0.98), rgba(40,15,17,1))"
                            .to_string()
                    }
                    GraphicsQuality::Low => {
                        "linear-gradient(180deg, rgba(171,104,83,0.05), rgba(0,0,0,0.08)), linear-gradient(180deg, rgba(73,45,45,0.97), rgba(43,29,31,1))"
                            .to_string()
                    }
                }
            }
            disabled=disabled
        >
            <Show when=move || settings.graphics_quality() != GraphicsQuality::Low>
                <span class="pointer-events-none absolute inset-x-2 top-[1px] h-px bg-gradient-to-r from-transparent via-[#ffc1ad]/40 to-transparent"></span>
            </Show>
            <span class="relative z-10">{children()}</span>
        </button>
    }
}

#[component]
pub fn FancyButton(
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let settings: SettingsContext = expect_context();

    view! {
        <button
            class=move || {
                let quality_class = match settings.graphics_quality() {
                    GraphicsQuality::High => {
                        "btn relative isolate overflow-hidden
                    tracking-[0.08em]
                    text-stone-100 font-extrabold text-shadow shadow-black/90
                    px-2 xl:px-3 rounded-[4px] xl:rounded-[6px]
                    text-sm xl:text-base
                    border border-[#6c5329]
                    shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(230,208,154,0.22),inset_0_-1px_0_rgba(0,0,0,0.45)]
                    before:pointer-events-none before:absolute before:inset-[1px]
                    before:rounded-[3px] xl:before:rounded-[5px]
                    before:border before:border-[#d5b16d]/18
                    before:bg-[linear-gradient(180deg,rgba(222,188,112,0.08),transparent_36%)]
                    bg-[#1b191d]
                    hover:border-[#a27f46]
                    hover:text-[#f3ead2]
                    hover:shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(244,225,181,0.28),inset_0_-1px_0_rgba(0,0,0,0.45)]
                    active:translate-y-[1px]
                    active:before:opacity-0 active:brightness-90
                    active:text-white
                    active:shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(26,17,10,0.95),inset_0_3px_5px_rgba(0,0,0,0.55),inset_0_-1px_0_rgba(0,0,0,0.22)]
                    disabled:text-zinc-500
                    disabled:border-[#4b4030]
                    disabled:opacity-60 disabled:shadow-none
                    disabled:before:hidden"
                    }
                    GraphicsQuality::Medium => {
                        "btn relative isolate overflow-hidden
                    tracking-[0.08em]
                    text-stone-100 font-extrabold text-shadow shadow-black/90
                    px-2 xl:px-3 rounded-[4px] xl:rounded-[6px]
                    text-sm xl:text-base
                    border border-[#6c5329]
                    bg-[#1b191d]
                    shadow-md
                    before:pointer-events-none before:absolute before:inset-[1px]
                    before:rounded-[3px] xl:before:rounded-[5px]
                    before:border before:border-[#d5b16d]/10
                    hover:border-[#a27f46] hover:text-[#f3ead2]
                    active:translate-y-[1px] active:brightness-90 active:text-white
                    disabled:text-zinc-500 disabled:border-[#4b4030] disabled:opacity-60
                    disabled:shadow-none"
                    }
                    GraphicsQuality::Low => {
                        "btn relative isolate overflow-hidden
                    tracking-[0.08em]
                    text-stone-100 font-extrabold
                    px-2 xl:px-3 rounded-[4px] xl:rounded-[6px]
                    text-sm xl:text-base
                    border border-[#5f5035]
                    before:pointer-events-none before:absolute before:inset-[1px]
                    before:rounded-[3px] xl:before:rounded-[5px]
                    before:border before:border-[#a88a53]/10
                    hover:border-[#8b7347] hover:text-[#efe1bf]
                    active:translate-y-[1px] active:brightness-95
                    disabled:text-zinc-500 disabled:border-[#4b4030] disabled:opacity-60"
                    }
                };
                format!("{quality_class} {}", class.unwrap_or_default())
            }
            style:background-image=move || {
                match settings.graphics_quality() {
                    GraphicsQuality::High => {
                        "linear-gradient(180deg, rgba(214,177,102,0.10), rgba(0,0,0,0.18)), linear-gradient(180deg, rgba(43,40,46,0.96), rgba(20,19,23,1))"
                            .to_string()
                    }
                    GraphicsQuality::Medium => {
                        "linear-gradient(180deg, rgba(170,140,84,0.08), rgba(0,0,0,0.08)), linear-gradient(180deg, rgba(45,42,48,0.98), rgba(23,22,26,1))"
                            .to_string()
                    }
                    GraphicsQuality::Low => {
                        "linear-gradient(180deg, rgba(176,145,86,0.05), rgba(0,0,0,0.08)), linear-gradient(180deg, rgba(56,53,58,0.97), rgba(31,30,34,1))"
                            .to_string()
                    }
                }
            }
            disabled=disabled
        >
            <Show when=move || settings.graphics_quality() != GraphicsQuality::Low>
                <span class="pointer-events-none absolute inset-x-2 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/55 to-transparent"></span>
            </Show>
            <Show when=move || settings.uses_heavy_effects()>
                <span class="pointer-events-none absolute left-[2px] top-[2px] bottom-[2px] w-px bg-gradient-to-b from-[#f0d79f]/35 via-transparent to-black/40"></span>
            </Show>
            <span class="relative z-10">{children()}</span>
        </button>
    }
}

#[component]
pub fn Toggle(
    #[prop(default = false)] initial: bool,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    mut toggle_callback: impl FnMut(bool) + 'static,
    children: Children,
) -> impl IntoView {
    let settings: SettingsContext = expect_context();
    let checked: RwSignal<bool> = RwSignal::new(initial);
    let switch_value = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        let new_value = !checked.get();
        checked.set(new_value);
        toggle_callback(new_value);
    };

    let toggle_class = move || {
        if checked.get() {
            match settings.graphics_quality() {
                GraphicsQuality::High => {
                    "text-zinc-100 
                    border-[#b28a4f] 
                    shadow-[0_6px_14px_rgba(0,0,0,0.52),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(255,231,183,0.30),inset_0_-1px_0_rgba(0,0,0,0.5)]"
                }
                GraphicsQuality::Medium => {
                    "text-zinc-100
                    border-[#a88449]
                    shadow-md
                    saturate-90"
                }
                GraphicsQuality::Low => {
                    "text-zinc-100
                    border-[#927444]
                    saturate-85"
                }
            }
        } else {
            match settings.graphics_quality() {
                GraphicsQuality::High => "opacity-60 shadow-none text-zinc-400 saturate-75",
                GraphicsQuality::Medium => {
                    "opacity-75 shadow-none text-zinc-400 saturate-50 border-[#5f5035]"
                }
                GraphicsQuality::Low => {
                    "opacity-80 shadow-none text-zinc-400 saturate-35 border-[#584a33]"
                }
            }
        }
    };

    view! {
        <button
            on:click=switch_value
            style:background-image=move || {
                match settings.graphics_quality() {
                    GraphicsQuality::High => {
                        "linear-gradient(180deg, rgba(214,177,102,0.08), rgba(0,0,0,0.18)), linear-gradient(180deg, rgba(42,39,45,0.95), rgba(18,17,22,1))"
                            .to_string()
                    }
                    GraphicsQuality::Medium => {
                        "linear-gradient(180deg, rgba(170,140,84,0.06), rgba(0,0,0,0.08)), linear-gradient(180deg, rgba(42,39,45,0.95), rgba(18,17,22,1))"
                            .to_string()
                    }
                    GraphicsQuality::Low => {
                        "linear-gradient(180deg, rgba(255,255,255,0.04), rgba(0,0,0,0.08)), linear-gradient(180deg, rgba(72,72,78,0.95), rgba(45,45,50,1))"
                            .to_string()
                    }
                }
            }
            class=move || {
                format!(
                    "btn relative {}
                    tracking-[0.08em]
                    px-2 xl:px-3
                    text-sm xl:text-base
                    font-extrabold {}
                    rounded-[4px] xl:rounded-[6px]
                    {}
                    shadow-md
                    hover:border-[#a27f46]
                    hover:text-[#f1e4c4]
                    active:text-white
                    disabled:text-zinc-500
                    disabled:border-[#4b4030]
                    disabled:opacity-60 disabled:shadow-none
                    disabled:before:hidden
                    transition-all duration-200
                    group
                    {}
                    ",
                    "isolate overflow-hidden",
                    if settings.uses_surface_effects() {
                        "text-shadow shadow-black/90"
                    } else {
                        ""
                    },
                    match settings.graphics_quality() {
                        GraphicsQuality::High => {
                            "before:pointer-events-none before:absolute before:inset-[1px] before:rounded-[3px] xl:before:rounded-[5px] before:border before:border-[#d5b16d]/16 before:bg-[linear-gradient(180deg,rgba(222,188,112,0.07),transparent_38%)]"
                        }
                        GraphicsQuality::Medium => {
                            "border border-[#6c5329] before:pointer-events-none before:absolute before:inset-[1px] before:rounded-[3px] xl:before:rounded-[5px] before:border before:border-[#d5b16d]/10"
                        }
                        GraphicsQuality::Low => {
                            "border border-[#5f5035] before:pointer-events-none before:absolute before:inset-[1px] before:rounded-[3px] xl:before:rounded-[5px] before:border before:border-[#a88a53]/10 hover:border-[#8b7347] hover:text-[#efe1bf] active:brightness-95 disabled:border-[#4b4030]"
                        }
                    },
                    toggle_class(),
                )
            }
            disabled=disabled
        >
            <Show when=move || settings.graphics_quality() != GraphicsQuality::Low>
                <span class="pointer-events-none absolute inset-x-2 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/45 to-transparent"></span>
            </Show>
            <Show when=move || settings.uses_heavy_effects()>
                <span
                    class="pointer-events-none absolute inset-0 rounded-[4px] xl:rounded-[6px] opacity-0 transition-opacity duration-200"
                    class:opacity-100=move || checked.get()
                    style="background: linear-gradient(180deg, rgba(255,255,255,0.04), transparent 42%, rgba(0,0,0,0.04));"
                ></span>
            </Show>
            <span class="relative z-1">{children()}</span>
        </button>
    }
}

#[component]
pub fn TabButton(
    children: Children,
    #[prop(into)] is_active: Signal<bool>,
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional)] title: Option<&'static str>,
) -> impl IntoView {
    let settings: SettingsContext = expect_context();
    let active_class = |active| {
        if active {
            "
            text-zinc-100
            border-[#9c7841]
            before:opacity-0 brightness-90
            shadow-[0_4px_10px_rgba(0,0,0,0.42),0_1px_0_rgba(26,17,10,0.95),inset_0_3px_5px_rgba(0,0,0,0.55),inset_0_-1px_0_rgba(0,0,0,0.22)]
            translate-y-[4px]
            "
        } else {
            "
            cursor-pointer
            border-[#6d5737]
            shadow-[0_4px_10px_rgba(0,0,0,0.42),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(216,186,122,0.12),inset_0_-1px_0_rgba(0,0,0,0.42)]
            hover:border-[#a27f46]
            hover:text-[#f1e4c4]
            hover:shadow-[0_4px_10px_rgba(0,0,0,0.42),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(244,225,181,0.2),inset_0_-1px_0_rgba(0,0,0,0.45)]
            active:shadow-[0_4px_10px_rgba(0,0,0,0.42),0_1px_0_rgba(26,17,10,0.95),inset_0_2px_3px_rgba(0,0,0,0.45),inset_0_-1px_0_rgba(0,0,0,0.18)]
            disabled:text-zinc-500
            disabled:border-[#4b4030]
            disabled:opacity-60
            "
        }
    };

    let disable_button = Signal::derive(move || {
        is_active.get() || disabled.map(|disabled| disabled.get()).unwrap_or_default()
    });

    view! {
        <button
            title=title
            style="
            background-size: auto, auto, 180px 180px;
            background-position: center, center, center;
            background-blend-mode: screen, normal, soft-light;
            "
            style:background-image=move || {
                match settings.graphics_quality() {
                    GraphicsQuality::High => {
                        "linear-gradient(180deg, rgba(214,177,102,0.10), rgba(0,0,0,0.18)), linear-gradient(180deg, rgba(43,40,46,0.96), rgba(20,19,23,1))"
                            .to_string()
                    }
                    GraphicsQuality::Medium => {
                        "linear-gradient(180deg, rgba(170,140,84,0.08), rgba(0,0,0,0.08)), linear-gradient(180deg, rgba(43,40,46,0.96), rgba(20,19,23,1))"
                            .to_string()
                    }
                    GraphicsQuality::Low => {
                        "linear-gradient(180deg, rgba(176,145,86,0.05), rgba(0,0,0,0.08)), linear-gradient(180deg, rgba(56,53,58,0.97), rgba(31,30,34,1))"
                            .to_string()
                    }
                }
            }
            class=move || {
                format!(
                    "btn relative {}
                    tracking-[0.08em]
                    inline-flex shrink-0 items-center justify-center
                    px-2 xl:px-3 py-1 xl:py-2
                    text-sm xl:text-base font-extrabold {}
                    border-t border-l border-r rounded-t-[6px] {}
                    transition-all duration-200
                    group
                    {}
                    ",
                    "isolate overflow-hidden",
                    if settings.uses_surface_effects() {
                        "text-shadow shadow-black/90"
                    } else {
                        ""
                    },
                    match settings.graphics_quality() {
                        GraphicsQuality::High => {
                            "before:pointer-events-none before:absolute before:inset-[1px] before:rounded-t-[5px] before:border-t before:border-l before:border-r before:border-[#d5b16d]/16 before:bg-[linear-gradient(180deg,rgba(222,188,112,0.07),transparent_38%)]"
                        }
                        GraphicsQuality::Medium => {
                            "before:pointer-events-none before:absolute before:inset-[1px] before:rounded-t-[5px] before:border-t before:border-l before:border-r before:border-[#d5b16d]/10"
                        }
                        GraphicsQuality::Low => {
                            "before:pointer-events-none before:absolute before:inset-[1px] before:rounded-t-[5px] before:border-t before:border-l before:border-r before:border-[#a88a53]/10"
                        }
                    },
                    active_class(is_active.get()),
                )
            }
            disabled=disable_button
        >
            <Show when=move || settings.graphics_quality() != GraphicsQuality::Low>
                <span class="pointer-events-none absolute inset-x-3 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/45 to-transparent"></span>
            </Show>
            <Show when=move || settings.uses_heavy_effects()>
                <span
                    class="pointer-events-none absolute inset-0 rounded-t-[6px] opacity-0 transition-opacity duration-200"
                    class:opacity-100=move || is_active.get()
                    style="background: linear-gradient(180deg, rgba(0,0,0,0.16), transparent 38%, rgba(0,0,0,0.06));"
                ></span>
            </Show>
            <span class="relative z-10">{children()}</span>
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
