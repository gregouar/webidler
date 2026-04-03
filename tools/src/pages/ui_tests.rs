use frontend::assets::img_asset;
use leptos::prelude::*;
use leptos_use::use_interval_fn;

use crate::header::HeaderMenu;

#[component]
pub fn UiTestsPage() -> impl IntoView {
    view! {
        <main class="my-0 mx-auto max-w-3xl text-center flex flex-col justify-around">
            <HeaderMenu />
            "Hello There"
            <Card class="w-3xl">
                <div class="w-full grid grid-cols-5 gap-2">
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
            style=format!(
                "
                background-image:
                    linear-gradient(180deg, rgba(214,177,102,0.08), rgba(0,0,0,0.18)),
                    linear-gradient(180deg, rgba(42,39,45,0.95), rgba(18,17,22,1)),
                    url('{}');
                background-size: auto, auto, 180px 180px;
                background-position: center, center, center;
                background-blend-mode: screen, normal, soft-light;
                ",
                img_asset("ui/dark_stone.webp"),
            )
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
