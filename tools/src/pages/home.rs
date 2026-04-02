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
                                Signal::derive(move || 1.0),
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
                                    <SegmentedCircularProgressBar
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
                                    </SegmentedCircularProgressBar>

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
                        *progress_value += rate * 0.1;
                    }
                    if remaining_time.get_untracked() == 0.0 && rate == 0.0 {
                        *progress_value = 1.0;
                    }
                });
            }
        },
        100,
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
                 
            ring-1 ring-zinc-900/80
            shadow-[0_4px_6px_rgba(0,0,0,0.25),inset_1px_1px_1px_rgba(255,255,255,0.06),inset_-1px_-1px_1px_rgba(0,0,0,0.15)]
            {} {} {}",
            class.unwrap_or_default(),
            if gap { "gap-1 xl:gap-2" } else { "" },
            if pad { "p-1 xl:p-3" } else { "" },
        )>{children()}</div>
    }
}

#[component]
pub fn SegmentedCircularProgressBar(
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
    const SEGMENT_COUNT: usize = 15;
    const BRASS_DARK: &str = "rgba(70, 49, 22, 0.98)";
    const BRASS_MID: &str = "rgba(146, 108, 52, 0.98)";
    const BRASS_BRIGHT: &str = "rgba(214, 171, 96, 0.98)";
    const BRASS_SHADOW: &str = "rgba(24, 14, 8, 0.92)";
    const SEGMENT_DARK: &str = "rgba(66, 28, 10, 0.96)";
    const SEGMENT_MID: &str = "rgba(150, 60, 14, 0.98)";
    const SEGMENT_BRIGHT: &str = "rgba(255, 165, 44, 0.98)";
    const SEGMENT_EDGE: &str = "rgba(255, 226, 178, 0.92)";
    const SEGMENT_LEAD: &str = "rgba(47, 23, 11, 0.95)";
    let _ = bar_color;
    const SEGMENT_GLOW: &str = "rgba(255, 126, 24, 0.78)";
    let _ = bar_width;

    let reset_overlay_style = RwSignal::new("opacity: 0;".to_string());
    let reset_icon_animation = RwSignal::new("");
    let display_segment_progress = RwSignal::new(0.0);
    let reset_segment_progress = RwSignal::new(0.0);
    let reset_armed = RwSignal::new(false);
    let lerp = |start: f64, end: f64, t: f64| start + (end - start) * t;
    let ease_out = |t: f64| 1.0 - (1.0 - t).powf(2.0);
    let ease_in = |t: f64| t.powf(1.35);
    let segment_state_from = move |progress: f64, index: usize| {
        let raw_fill = (progress - index as f64).clamp(0.0, 1.0);
        let glow_fill = ease_out(raw_fill);
        let depth_fill = ease_in(raw_fill);
        (raw_fill, glow_fill, depth_fill)
    };

    Effect::new(move |_| {
        display_segment_progress.set(value.get().clamp(0.0, 1.0) * SEGMENT_COUNT as f64);
    });

    Effect::new(move |_| {
        if reset.get() && !reset_armed.get() {
            reset_armed.set(true);

            if !disabled.get_untracked() {
                reset_segment_progress.set(display_segment_progress.get_untracked());
                reset_overlay_style
                    .set("opacity: 1; animation: circular-progress-bar-fade-out 0.5s linear; animation-fill-mode: both;".to_string());
                reset_icon_animation.set(
                    "animation: circular-progress-bar-glow 0.5s ease; animation-fill-mode: both;",
                );

                // Trick to reset animation by removing it when ended
                set_timeout(
                    move || {
                        reset_overlay_style.set("opacity: 0;".to_string());
                        reset_icon_animation.set("");
                    },
                    std::time::Duration::from_millis(500),
                );
            }
        } else if !reset.get() {
            reset_armed.set(false);
        }
    });

    let segment_ring = move |progress_signal: RwSignal<f64>,
                             container_style: RwSignal<String>,
                             animated: bool| {
        view! {
            <div
                class="absolute inset-[5%] rounded-full will-change-opacity"
                style=move || container_style.get()
            >
                {(0..SEGMENT_COUNT)
                    .map(|index| {
                        let angle = (index as f64 + 0.5) * (360.0 / SEGMENT_COUNT as f64);
                        view! {
                            <div
                                class="absolute inset-0 flex items-start justify-center"
                                style=format!("transform: rotate({angle}deg);")
                            >
                                <div
                                    class="relative mt-[1.2%] h-[13%] w-[22%] origin-center"
                                    class:brightness-60=move || disabled.get()
                                    style=format!(
                                        "clip-path: polygon(16% 0%, 84% 0%, 72% 100%, 28% 100%);
                                         border-radius: 4px 4px 10px 10px;
                                         background: linear-gradient(135deg,
                                            rgba(255,244,222,0.06) 0%,
                                            rgba(180,104,36,0.34) 16%,
                                            rgba(114,49,16,0.92) 40%,
                                            rgba(74,31,11,0.98) 72%,
                                            rgba(36,14,6,1.00) 100%);
                                         border: 1px solid rgba(255,226,178,0.16);
                                         box-shadow:
                                            inset 1px 1px 2px rgba(255,244,222,0.08),
                                            inset -2px -3px 5px rgba(24,8,2,0.72);"
                                    )
                                >
                                    <div
                                        class=if animated {
                                            "absolute inset-0 transition-opacity duration-200 ease-linear"
                                        } else {
                                            "absolute inset-0"
                                        }
                                        style=move || {
                                            let progress = progress_signal.get();
                                            let (_, glow_fill, _) = segment_state_from(progress, index);
                                            format!(
                                                "clip-path: polygon(16% 0%, 84% 0%, 72% 100%, 28% 100%);
                                                 border-radius: 4px 4px 10px 10px;
                                                 background: linear-gradient(135deg,
                                                    rgba(255,244,212,0.16) 0%,
                                                    rgba(255,192,78,0.88) 18%,
                                                    rgba(255,140,28,0.94) 42%,
                                                    rgba(173,64,12,0.92) 76%,
                                                    rgba(78,27,8,0.68) 100%);
                                                 border: 1px solid rgba(255,228,188,0.38);
                                                 opacity: {};",
                                                lerp(0.0, 1.0, glow_fill),
                                            )
                                        }
                                    ></div>
                                    <div
                                        class=if animated {
                                            "absolute left-[18%] top-[10%] h-[18%] w-[42%] transition-opacity duration-200 ease-linear"
                                        } else {
                                            "absolute left-[18%] top-[10%] h-[18%] w-[42%]"
                                        }
                                        style=move || {
                                            let progress = progress_signal.get();
                                            let (_, glow_fill, _) = segment_state_from(progress, index);
                                            format!(
                                                "clip-path: polygon(0% 100%, 64% 0%, 100% 14%, 34% 100%);
                                                 background: linear-gradient(135deg,
                                                    rgba(255,252,246,0.86),
                                                    rgba(255,245,224,0.18) 48%,
                                                    rgba(255,244,222,0.00));
                                                 opacity: {};",
                                                lerp(0.0, 0.58, glow_fill),
                                            )
                                        }
                                    ></div>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        }
    };

    let live_ring_style = RwSignal::new(String::new());

    view! {
        <div class="circular-progress-bar">
            <div
                class="relative w-full h-full aspect-square rounded-full overflow-hidden
                bg-radial from-stone-800 to-zinc-950 to-70%"
                style="contain: strict;"
            >

                // <div
                // class="absolute rounded-full border border-[#120d0a] shadow-[inset_0_2px_3px_rgba(255,255,255,0.04),inset_0_-10px_16px_rgba(0,0,0,0.82)]"

                // ></div>

                <div
                    class="absolute inset-0 rounded-full pointer-events-none"
                    style=format!(
                        "box-shadow:
                            inset 0 0 0 2px {BRASS_DARK},
                            inset 0 1px 0 2px rgba(255,233,185,0.38),
                            inset 0 -2px 0 2px rgba(60,40,16,0.92);",
                    )
                ></div>

                {segment_ring(display_segment_progress, live_ring_style, true)}
                {segment_ring(reset_segment_progress, reset_overlay_style, false)}

                <div
                    class="absolute top-1/2 start-1/2 transform -translate-y-1/2 -translate-x-1/2
                    will-change-[filter,transform] transition-[filter,transform] duration-500"
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
            border border-neutral-950
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
                    border border-neutral-950 rounded 
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
