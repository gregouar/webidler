use frontend::assets::img_asset;
use leptos::prelude::*;
use leptos_use::use_interval_fn;
use shared::data::monster::MonsterRarity;

use crate::header::HeaderMenu;

#[component]
pub fn UiTestsPage() -> impl IntoView {
    view! {
        <main class="my-0 mx-auto max-w-3xl text-center flex flex-col justify-around">
            <HeaderMenu />
            "Hello There"
            <MenuButton>"Passives"</MenuButton>
            <MenuButton>"Ui Tests"</MenuButton>
            <Card class="w-2xl">
                <div class="my-4 w-full flex justify-center">
                    <div class="w-sm">
                        <CharacterPortrait
                            image_uri="monsters/pirate_pistol.webp".into()
                            character_name="Rat".into()
                            rarity=MonsterRarity::Normal
                        />
                    </div>
                </div>
                <div class="w-full grid grid-cols-4 gap-2">
                    {(0..3)
                        .map(|_| {
                            let trigger_reset_progress = RwSignal::new(false);
                            let reset_progress = Signal::derive(move || {
                                trigger_reset_progress.get()
                            });
                            let progress_value = predictive_cooldown(
                                Signal::derive(move || 2.0),
                                reset_progress,
                                Signal::derive(move || false),
                                0.0,
                            );
                            Effect::new(move || {
                                if progress_value.get() >= 1.0 {
                                    trigger_reset_progress.set(true)
                                } else {
                                    trigger_reset_progress.set(false)
                                }
                            });

                            view! {
                                <div class="flex flex-col gap-1">
                                    <CircularProgressBar
                                        bar_color="oklch(55.5% 0.163 48.998)"
                                        value=progress_value
                                        reset=reset_progress
                                        bar_width=4
                                    >
                                        <img
                                            draggable="false"
                                            src="assets/images/skills/attack.svg"
                                            alt="attack"
                                            class="w-full h-full flex-no-shrink fill-current
                                            xl:drop-shadow-[0px_4px_oklch(13% 0.028 261.692)] invert"
                                        />
                                    </CircularProgressBar>

                                    <div class="flex justify-around">
                                        <Toggle toggle_callback=|_| {}>
                                            <span class="inline xl:hidden">"A"</span>
                                            <span class="hidden xl:inline font-variant:small-caps">
                                                "Auto"
                                            </span>
                                        </Toggle>
                                        <FancyButton>
                                            <span class="text-base xl:text-2xl">"+"</span>
                                        </FancyButton>
                                    </div>
                                </div>
                            }
                        })
                        .collect::<Vec<_>>()}
                </div>
            </Card>
        </main>
    }
}

pub fn predictive_cooldown(
    remaining_time: Signal<f64>,
    reset: Signal<bool>,
    disabled: Signal<bool>,
    starting_value: f64,
) -> RwSignal<f64> {
    let progress_value = RwSignal::new(starting_value);
    let rate = RwSignal::new(0.0);

    Effect::new(move || {
        let remaining_time = remaining_time.get();
        if remaining_time > 0.0 {
            let remaining: f64 = (1.0f64 - progress_value.get_untracked()).clamp(0.0, 1.0);
            rate.set(remaining / remaining_time);
        }
    });

    Effect::new(move || {
        if reset.get() {
            progress_value.set(0.0);
        }
    });

    use_interval_fn(
        move || {
            let rate = rate.get_untracked();
            if !disabled.get_untracked() && rate > 0.0 {
                progress_value.update(|progress_value| {
                    if *progress_value < 1.2 {
                        *progress_value += rate * 0.2;
                    }
                    if remaining_time.get_untracked() == 0.0 && rate == 0.0 {
                        *progress_value = 1.0;
                    }
                });
            }
        },
        200,
    );

    progress_value
}

#[component]
pub fn Card(
    #[prop(optional)] class: Option<&'static str>,
    #[prop(default = true)] gap: bool,
    #[prop(default = true)] pad: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=format!(
                "max-h-full flex flex-col relative overflow-hidden
                bg-zinc-800
                border border-[#6c5734]/45
                shadow-[0_6px_15px_rgba(0,0,0,0.35),inset_2px_2px_1px_rgba(255,255,255,0.06),inset_-2px_-2px_1px_rgba(0,0,0,0.15)]
                {} {} {}",
                class.unwrap_or_default(),
                if gap { "gap-1 xl:gap-2" } else { "" },
                if pad { "p-1 xl:p-3" } else { "" },
            )
            style=format!(
                "
                clip-path: polygon(12px 0, calc(100% - 12px) 0, 100% 12px, 100% calc(100% - 12px), calc(100% - 12px) 100%, 12px 100%, 0 calc(100% - 12px), 0 12px);
                background-image: url('{}'); 
                background-blend-mode: multiply;
                ",
                img_asset("ui/dark_stone.webp"),
            )
        >
            <div
                class="pointer-events-none absolute inset-[1px] border border-white/6"
                style="clip-path: polygon(11px 0, calc(100% - 11px) 0, 100% 11px, 100% calc(100% - 11px), calc(100% - 11px) 100%, 11px 100%, 0 calc(100% - 11px), 0 11px);"
            ></div>

            <div class="relative z-10 flex max-h-full flex-col">{children()}</div>
        </div>
    }
}

#[component]
pub fn CircularProgressBar(
    // Percent value, must be between 0 and 100.
    #[prop(into)] value: Signal<f64>,
    bar_color: &'static str,
    bar_width: u8,
    // Instant reset
    #[prop(into,default = Signal::derive(|| false))] reset: Signal<bool>,
    #[prop(into,default = Signal::derive(|| false))] disabled: Signal<bool>,
    // Inside the circular bar
    children: Children,
) -> impl IntoView {
    let reset_icon_animation = RwSignal::new("");
    let active_buffer = RwSignal::new(false);
    let front_progress = RwSignal::new(value.get_untracked().clamp(0.0, 1.0) * 100.0);
    let back_progress = RwSignal::new(0.0);

    Effect::new(move |_| {
        if reset.get() {
            if !disabled.get_untracked() {
                reset_icon_animation.set(
                    "animation: circular-progress-bar-glow 0.5s ease; animation-fill-mode: both;",
                );
            }

            if active_buffer.get_untracked() {
                active_buffer.set(false);

                set_timeout(
                    move || {
                        reset_icon_animation.set("");
                        back_progress.set(0.0);
                    },
                    std::time::Duration::from_millis(500),
                );
            } else {
                active_buffer.set(true);

                set_timeout(
                    move || {
                        reset_icon_animation.set("");
                        front_progress.set(0.0);
                    },
                    std::time::Duration::from_millis(500),
                );
            }
        } else {
            let progress = if reset.get() {
                0.0
            } else {
                value.get().clamp(0.0, 1.0) * 100.0
            };

            if active_buffer.get() {
                back_progress.set(progress);
            } else {
                front_progress.set(progress);
            }
        }
    });

    view! {
        <div class="circular-progress-bar">
            <div
                class="relative w-full h-full aspect-square rounded-full overflow-hidden
                xl:drop-shadow-[0_0_5px_rgba(0,0,0,0.5)]
                border border-[#7a6137]
                shadow-[0_8px_18px_rgba(0,0,0,0.48),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(230,208,154,0.22),inset_0_-1px_0_rgba(0,0,0,0.45)]"
                style=
                    "
                    contain: strict;
                    background-image:
                        linear-gradient(180deg, rgba(214,177,102,0.08), rgba(0,0,0,0.18)),
                        linear-gradient(180deg, rgba(34,32,37,0.96), rgba(15,14,18,1));
                    background-size: auto, auto;
                    background-position: center, center;
                    background-blend-mode: screen, normal;
                    "
            >
                <div class="pointer-events-none absolute inset-[1px] rounded-full border border-[#d5b16d]/18"></div>
                <div class="pointer-events-none absolute inset-0 rounded-full bg-[radial-gradient(circle_at_50%_10%,rgba(229,194,120,0.08),transparent_38%),radial-gradient(circle_at_50%_100%,rgba(0,0,0,0.22),transparent_44%)]"></div>

                <div
                    class="absolute inset-0 will-change-(--progress) will-change-opacity
                    transition-circular-progress-bar"
                    class:opacity-0=move || disabled.get()
                    class:fade-out-circular-progress-bar=move || active_buffer.get()
                    style=format!(
                            "
                            background: conic-gradient(
                                {bar_color} var(--progress),
                                transparent var(--progress) 100%
                            );
                            ",
                        )
                    style:--progress=move || format!("{}%", front_progress.get())

                ></div>

                <div
                    class="absolute inset-0 will-change-(--progress) will-change-opacity
                    transition-circular-progress-bar"
                    class:opacity-0=move || disabled.get()
                    class:fade-out-circular-progress-bar=move || !active_buffer.get()
                    style=format!(
                            "
                            background: conic-gradient(
                                {bar_color} var(--progress),
                                transparent var(--progress) 100%
                            );
                            ",
                        )
                    style:--progress=move || format!("{}%", back_progress.get())
                ></div>

                <div class=format!(
                    "absolute inset-{} xl:inset-{bar_width} rounded-full
                        bg-radial from-stone-600 to-zinc-950 to-70% 
                        border border-[#6d532e]/70 shadow-[inset_0_2px_6px_rgba(0,0,0,0.55),inset_0_1px_0_rgba(236,210,148,0.14),0_1px_2px_rgba(0,0,0,0.35)]",
                    bar_width / 2,
                )>
                </div>

                // Icon
                <div
                    class="absolute top-1/2 start-1/2 transform -translate-y-1/2 -translate-x-1/2
                    scale-120 xl:drop-shadow-[0_2px_0px_rgba(0,0,0,0.5)]
                    will-change-transform transition-transform duration-500"
                    style=reset_icon_animation
                    class:brightness-50=move || disabled.get()
                >
                    {children()}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn MenuButton(
    #[prop(optional, into)] disabled: Option<Signal<bool>>,
    #[prop(optional)] button_type: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <button
            class="btn relative isolate overflow-hidden
            tracking-[0.08em]
            text-stone-100 font-extrabold text-shadow-lg/50 shadow-black/90
            py-1 xl:py-2 px-2 xl:px-4 rounded-[4px] xl:rounded-[6px]
            text-sm xl:text-base
            border border-[#7a6137]
            shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(230,208,154,0.22),inset_0_-1px_0_rgba(0,0,0,0.45)]
            before:pointer-events-none before:absolute before:inset-[1px]
            before:rounded-[3px] xl:before:rounded-[5px]
            before:border before:border-[#d5b16d]/18
            before:bg-[linear-gradient(180deg,rgba(222,188,112,0.08),transparent_36%)]
            hover:border-[#a27f46]
            hover:text-[#f3ead2]
            hover:shadow-[0_6px_14px_rgba(0,0,0,0.5),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(244,225,181,0.28),inset_0_-1px_0_rgba(0,0,0,0.45)]
            active:translate-y-[1px]
            active:shadow-[0_2px_6px_rgba(0,0,0,0.55),0_1px_0_rgba(26,17,10,0.95),inset_0_2px_3px_rgba(0,0,0,0.45)]
            w-auto
            disabled:text-zinc-500
            disabled:border-[#4b4030]
            disabled:opacity-60 disabled:shadow-none
            disabled:before:hidden"
            style="
            background-image:
            linear-gradient(180deg, rgba(214,177,102,0.10), rgba(0,0,0,0.18)),
            linear-gradient(180deg, rgba(43,40,46,0.96), rgba(20,19,23,1));
            background-size: auto, auto, 180px 180px;
            background-position: center, center, center;
            background-blend-mode: screen, normal, soft-light;
            "
            type=button_type
            disabled=disabled
        >
            <span class="pointer-events-none absolute inset-x-2 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/55 to-transparent"></span>
            <span class="pointer-events-none absolute left-[2px] top-[2px] bottom-[2px] w-px bg-gradient-to-b from-[#f0d79f]/35 via-transparent to-black/40"></span>
            <span class="relative z-10">{children()}</span>
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
            class="btn relative isolate overflow-hidden
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
            hover:shadow-[0_6px_14px_rgba(0,0,0,0.5),0_1px_0_rgba(34,10,10,0.95),inset_0_1px_0_rgba(255,220,198,0.24),inset_0_-1px_0_rgba(0,0,0,0.45)]
            active:translate-y-[1px]
            active:shadow-[0_2px_6px_rgba(0,0,0,0.55),0_1px_0_rgba(34,10,10,0.95),inset_0_2px_3px_rgba(0,0,0,0.45)]
            disabled:text-zinc-500
            disabled:border-[#4f312d]
            disabled:opacity-60 disabled:shadow-none
            disabled:before:hidden"
            style="
            background-image:
            linear-gradient(180deg, rgba(230,164,125,0.14), rgba(0,0,0,0.18)),
            linear-gradient(180deg, rgba(72,28,26,0.98), rgba(35,11,13,1));
            background-size: auto, auto, 180px 180px;
            background-position: center, center, center;
            background-blend-mode: screen, normal, soft-light;
            "
            disabled=disabled
        >
            <span class="pointer-events-none absolute inset-x-2 top-[1px] h-px bg-gradient-to-r from-transparent via-[#ffc1ad]/40 to-transparent"></span>
            <span class="relative z-10">{children()}</span>
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
            class="btn relative isolate overflow-hidden
            tracking-[0.08em]
            text-stone-100 font-extrabold text-shadow shadow-black/90
            px-2 xl:px-3 rounded-[4px] xl:rounded-[6px]
            text-sm xl:text-base
            border border-[#7a6137]
            shadow-[0_4px_10px_rgba(0,0,0,0.45),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(230,208,154,0.22),inset_0_-1px_0_rgba(0,0,0,0.45)]
            before:pointer-events-none before:absolute before:inset-[1px]
            before:rounded-[3px] xl:before:rounded-[5px]
            before:border before:border-[#d5b16d]/18
            before:bg-[linear-gradient(180deg,rgba(222,188,112,0.08),transparent_36%)]
            bg-[#1b191d]
            hover:border-[#a27f46]
            hover:text-[#f3ead2]
            hover:shadow-[0_6px_14px_rgba(0,0,0,0.5),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(244,225,181,0.28),inset_0_-1px_0_rgba(0,0,0,0.45)]
            active:translate-y-[1px]
            active:shadow-[0_2px_6px_rgba(0,0,0,0.55),0_1px_0_rgba(26,17,10,0.95),inset_0_2px_3px_rgba(0,0,0,0.45)]
            disabled:text-zinc-500
            disabled:border-[#4b4030]
            disabled:opacity-60 disabled:shadow-none
            disabled:before:hidden"
            style=format!(
                "
                background-image:
                    linear-gradient(180deg, rgba(214,177,102,0.10), rgba(0,0,0,0.18)),
                    linear-gradient(180deg, rgba(43,40,46,0.96), rgba(20,19,23,1));
                background-size: auto, auto, 180px 180px;
                background-position: center, center, center;
                background-blend-mode: screen, normal, soft-light;
                ",
            )
            disabled=disabled
        >
            <span class="pointer-events-none absolute inset-x-2 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/55 to-transparent"></span>
            <span class="pointer-events-none absolute left-[2px] top-[2px] bottom-[2px] w-px bg-gradient-to-b from-[#f0d79f]/35 via-transparent to-black/40"></span>
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
    let checked: RwSignal<bool> = RwSignal::new(initial);
    let switch_value = move |ev: web_sys::MouseEvent| {
        ev.stop_propagation();
        let new_value = !checked.get();
        checked.set(new_value);
        toggle_callback(new_value);
    };

    let toggle_class = move || {
        if checked.get() {
            "text-[#f6edd8] border-[#b28a4f] shadow-[0_6px_14px_rgba(0,0,0,0.52),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(255,231,183,0.30),inset_0_-1px_0_rgba(0,0,0,0.5)]"
        } else {
            "text-zinc-400 border-[#65533a] shadow-[0_4px_10px_rgba(0,0,0,0.42),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(216,186,122,0.12),inset_0_-1px_0_rgba(0,0,0,0.42)]"
        }
    };

    view! {
        <button
            on:click=switch_value
            style="
            background-image:
            linear-gradient(180deg, rgba(214,177,102,0.08), rgba(0,0,0,0.18)),
            linear-gradient(180deg, rgba(42,39,45,0.95), rgba(18,17,22,1));
            background-size: auto, auto;
            background-position: center, center;
            background-blend-mode: screen, normal;
            "
            class=move || {
                format!(
                    "btn relative isolate overflow-hidden
                    tracking-[0.08em]
                    px-2 xl:px-3
                    text-sm xl:text-base
                    font-extrabold text-shadow shadow-black/90
                    rounded-[4px] xl:rounded-[6px]
                    before:pointer-events-none before:absolute before:inset-[1px]
                    before:rounded-[3px] xl:before:rounded-[5px]
                    before:border before:border-[#d5b16d]/16
                    before:bg-[linear-gradient(180deg,rgba(222,188,112,0.07),transparent_38%)]
                    hover:border-[#a27f46]
                    hover:text-[#f1e4c4]
                    active:translate-y-[1px]
                    active:shadow-[0_2px_6px_rgba(0,0,0,0.55),0_1px_0_rgba(26,17,10,0.95),inset_0_2px_3px_rgba(0,0,0,0.45)]
                    disabled:text-zinc-500
                    disabled:border-[#4b4030]
                    disabled:opacity-60 disabled:shadow-none
                    disabled:before:hidden
                    transition-all duration-200
                    group
                    {}
                    ",
                    toggle_class(),
                )
            }
            disabled=disabled
        >
            <span class="pointer-events-none absolute inset-x-2 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/45 to-transparent"></span>
            <span
                class="pointer-events-none absolute inset-0 rounded-[4px] xl:rounded-[6px] opacity-0 transition-opacity duration-200"
                class:opacity-100=move || checked.get()
                style="background: radial-gradient(circle at 50% 0%, rgba(229, 194, 120, 0.18), transparent 58%);"
            ></span>
            <span class="relative z-10">{children()}</span>
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
    let active_class = |active| {
        if active {
            "
            text-[#f5ecd6]
            border-[#b28a4f]
            shadow-[0_2px_6px_rgba(0,0,0,0.5),0_1px_0_rgba(26,17,10,0.95),inset_0_2px_4px_rgba(0,0,0,0.42)]
            translate-y-[2px]
            "
        } else {
            "
            cursor-pointer
            text-zinc-300
            border-[#6d5737]
            shadow-[0_4px_10px_rgba(0,0,0,0.42),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(216,186,122,0.12),inset_0_-1px_0_rgba(0,0,0,0.42)]
            hover:border-[#a27f46]
            hover:text-[#f1e4c4]
            hover:shadow-[0_6px_14px_rgba(0,0,0,0.5),0_1px_0_rgba(26,17,10,0.95),inset_0_1px_0_rgba(244,225,181,0.2),inset_0_-1px_0_rgba(0,0,0,0.45)]
            active:shadow-[0_2px_6px_rgba(0,0,0,0.55),0_1px_0_rgba(26,17,10,0.95),inset_0_2px_3px_rgba(0,0,0,0.45)]
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
            background-image:
            linear-gradient(180deg, rgba(214,177,102,0.10), rgba(0,0,0,0.18)),
            linear-gradient(180deg, rgba(43,40,46,0.96), rgba(20,19,23,1));
            background-size: auto, auto, 180px 180px;
            background-position: center, center, center;
            background-blend-mode: screen, normal, soft-light;
            "
            class=move || {
                format!(
                    "btn relative isolate overflow-hidden
                    tracking-[0.08em]
                    flex-1
                    px-2 xl:px-3 py-1 xl:py-2
                    text-sm xl:text-base
                    font-extrabold text-shadow shadow-black/90
                    border-t border-l border-r rounded-t-[6px]
                    before:pointer-events-none before:absolute before:inset-[1px]
                    before:rounded-t-[5px]
                    before:border-t before:border-l before:border-r before:border-[#d5b16d]/16
                    before:bg-[linear-gradient(180deg,rgba(222,188,112,0.07),transparent_38%)]
                    transition-all duration-200
                    group
                    {}
                    ",
                    active_class(is_active.get()),
                )
            }
            disabled=disable_button
        >
            <span class="pointer-events-none absolute inset-x-3 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/45 to-transparent"></span>
            <span
                class="pointer-events-none absolute inset-0 rounded-t-[6px] opacity-0 transition-opacity duration-200"
                class:opacity-100=move || is_active.get()
                style="background: radial-gradient(circle at 50% 0%, rgba(229, 194, 120, 0.18), transparent 58%);"
            ></span>
            <span class="relative z-10">{children()}</span>
        </button>
    }
}

#[component]
pub fn CharacterPortrait(
    image_uri: String,
    character_name: String,
    #[prop(default = MonsterRarity::Normal)] rarity: MonsterRarity,
) -> impl IntoView {
    let (accent_class, shimmer_effect, fixture_class) = match rarity {
        MonsterRarity::Normal => (
            "
            border-[#7f6744]
            before:border-[#d0b173]/12
            after:border-[#5a4427]/45
            ",
            "",
            "
            border-[#b89458]
            bg-[linear-gradient(180deg,rgb(214,184,126),rgb(111,78,33))]
            ",
        ),

        MonsterRarity::Champion => (
            "
            border-[#4f5fbe]
            before:border-[#97a7ff]/14
            after:border-[#2c356d]/55
            ",
            "champion-shimmer",
            "
            border-[#7a87d8]
            bg-[linear-gradient(180deg,rgb(154,170,255),rgb(57,69,137))]
            ",
        ),

        MonsterRarity::Boss => (
            "
            border-[#ab473c]
            before:border-[#f2a18c]/20
            after:border-[#6d2119]/60
            ",
            "boss-shimmer",
            "
            border-[#d77a68]
            bg-[linear-gradient(180deg,rgb(247,167,145),rgb(116,38,30))]
            ",
        ),
    };

    view! {
        <style>"color: #2e2926;"</style>
        <div class="flex items-center justify-center w-full h-full relative p-1 xl:p-2">
            <div
                class=format!(
                    "w-full h-full relative isolate
                    border-[1.5px] xl:border-2
                    shadow-[0_6px_12px_rgba(0,0,0,0.34),0_1px_0_rgba(23,15,8,0.82),inset_0_1px_0_rgba(243,221,173,0.12),inset_0_-1px_0_rgba(0,0,0,0.2)]
                    before:pointer-events-none before:absolute before:inset-[1px]
                    before:border before:bg-[linear-gradient(180deg,rgba(228,194,119,0.06),transparent_28%)]
                    after:pointer-events-none after:absolute after:inset-[4px]
                    after:border-[1px]
                    {} {}",
                    accent_class,
                    shimmer_effect,
                )
                style=format!(
                    "
                    background-image:
                        linear-gradient(180deg, rgba(214,177,102,0.13), rgba(0,0,0,0.2)),
                        linear-gradient(180deg, rgba(68,49,28,0.9), rgba(26,20,16,0.97));
                    background-size: auto, auto;
                    background-position: center, center;
                    background-blend-mode: screen, normal;
                    ",
                )
            >
                <div class="pointer-events-none absolute inset-x-6 top-[1px] h-px bg-gradient-to-r from-transparent via-[#f0d79f]/28 to-transparent"></div>
                <div class="pointer-events-none absolute inset-0 bg-[linear-gradient(90deg,rgba(0,0,0,0.12),transparent_12%,transparent_88%,rgba(0,0,0,0.15))]"></div>

                <div
                    class="h-full overflow-hidden border border-black/40 bg-[#1c1714] shadow-[inset_0_1px_0_rgba(255,241,208,0.04),inset_0_0_8px_rgba(0,0,0,0.24)]"
                    style=format!(
                        "
                        background-image:
                            linear-gradient(180deg, rgba(255,236,194,0.05), rgba(0,0,0,0.1)),
                            url('{}');
                        background-size: auto, cover;
                        background-position: center, center;
                        ",
                        img_asset("ui/paper_background.webp"),
                    )
                >
                    <div class="pointer-events-none absolute inset-0 border-[2px] xl:border-[3px] border-[#2a1e19]/68"></div>
                    <div class="pointer-events-none absolute inset-0 z-10 bg-[radial-gradient(circle_at_50%_15%,rgba(255,241,210,0.04),transparent_34%),linear-gradient(180deg,transparent_68%,rgba(0,0,0,0.14))]"></div>
                    <img
                        draggable="false"
                        src=img_asset(&image_uri)
                        alt=character_name
                        class="object-cover h-full w-full transition-all duration-[5s]"
                    />
                </div>

                <div class=format!(
                    "pointer-events-none absolute -top-[5px] -left-[5px] h-[12px] w-[12px]
                     rotate-315 border shadow-[0_2px_3px_rgba(0,0,0,0.3),inset_0_1px_0_rgba(255,241,209,1.0)] {}",
                    fixture_class,
                )></div>
                <div class=format!(
                    "pointer-events-none absolute -top-[5px] -right-[5px] h-[12px] w-[12px]
                     rotate-315 border shadow-[0_2px_3px_rgba(0,0,0,0.3),inset_0_1px_0_rgba(255,241,209,1.0)] {}",
                    fixture_class,
                )></div>
                <div class=format!(
                    "pointer-events-none absolute -bottom-[5px] -left-[5px] h-[12px] w-[12px]
                     rotate-315 border shadow-[0_2px_3px_rgba(0,0,0,0.3),inset_0_1px_0_rgba(255,241,209,1.0)] {}",
                    fixture_class,
                )></div>
                <div class=format!(
                    "pointer-events-none absolute -bottom-[5px] -right-[5px] h-[12px] w-[12px]
                     rotate-315 border shadow-[0_2px_3px_rgba(0,0,0,0.3),inset_0_1px_0_rgba(255,241,209,1.0)] {}",
                    fixture_class,
                )></div>

            </div>
        </div>
    }
}
