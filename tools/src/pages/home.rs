// use frontend::components::ui::progress_bars::CircularProgressBar;
use leptos::prelude::*;
use leptos_use::use_interval_fn;

use crate::header::HeaderMenu;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        // <MenuButton on:click=navigate_to_leaderboard>"Leaderboard"</MenuButton>
        <main class="my-0 mx-auto max-w-3xl text-center flex flex-col justify-around">
            <HeaderMenu />
            "Hello There"
            <Card class="w-3xl">
                <div class="w-full grid grid-cols-5 gap-2">
                    {(0..20)
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
        <div class=format!(
            "max-h-full flex flex-col relative
            bg-zinc-800 
            rounded-[6px] xl:rounded-[8px]
                 
            ring-1 ring-zinc-700/50
            shadow-[0_4px_6px_rgba(0,0,0,0.25),inset_1px_1px_1px_rgba(255,255,255,0.06),inset_-1px_-1px_1px_rgba(0,0,0,0.15)]
            {} {} {}",
            class.unwrap_or_default(),
            if gap { "gap-1 xl:gap-2" } else { "" },
            if pad { "p-1 xl:p-3" } else { "" },
        )>{children()}</div>
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
                class="relative w-full h-full aspect-square rounded-full bg-stone-900 overflow-hidden"
                style="contain: strict;"
            >
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

                // Hole in the middle
                <div class=format!(
                    "absolute inset-{} xl:inset-{bar_width} rounded-full
                        bg-radial from-stone-600 to-zinc-950 to-70%",
                    bar_width / 2,
                )></div>

                // Icon
                <div
                    class="absolute top-1/2 start-1/2 transform -translate-y-1/2 -translate-x-1/2
                    scale-125
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
            class="btn
            tracking-wide
            text-white font-extrabold text-shadow shadow-neutral-950
            px-2 xl:px-3 rounded shadow-md
            text-sm xl:text-base 
            border border-zinc-700/50
            bg-gradient-to-t from-zinc-900 to-zinc-800 
            overflow-hidden
            hover:bg-gradient-to-tr hover:from-zinc-900 hover:to-neutral-700 
            active:bg-gradient-to-t active:from-zinc-900 active:to-zinc-950 
            active:translate-y-[1px]
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
                    tracking-wide
                    px-2 xl:px-3
                    text-sm xl:text-base 
                    font-extrabold text-shadow shadow-neutral-950
                    border border-zinc-700/50 rounded 
                    bg-gradient-to-t from-zinc-900 to-zinc-800 
                    hover:bg-gradient-to-tr hover:from-zinc-900 hover:to-neutral-700
                    active:bg-gradient-to-t active:from-zinc-900 active:to-zinc-950
                    active:translate-y-[1px]
                    disabled:from-zinc-700 disabled:to-zinc-600
                    disabled:text-zinc-400
                    disabled:opacity-60 disabled:shadow-none
                    transition-all duration-200
                    relative
                    group
                    {}
                    ",
                    toggle_class(),
                )
            }
            disabled=disabled
        >
            {children()}
        </button>
    }
}
