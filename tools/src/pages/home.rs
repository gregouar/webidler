use frontend::components::ui::progress_bars::predictive_cooldown;
use leptos::prelude::*;

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
                                Signal::derive(move || 5.0),
                                reset_progress,
                                Signal::derive(move || false),
                                0.0,
                            );
                            Effect::new(move || {
                                if progress_value.get() >= 0.99 {
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
    const SEGMENT_GLOW: &str = "rgba(255, 126, 24, 0.78)";
    let _ = bar_width;

    let reset_bar_animation = RwSignal::new("opacity: 0;");
    let reset_icon_animation = RwSignal::new("");
    let enable_transition = RwSignal::new(true);
    let active_segments = Signal::derive(move || {
        ((value.get().clamp(0.0, 1.0) * SEGMENT_COUNT as f64).round() as usize).min(SEGMENT_COUNT)
    });

    Effect::new(move |_| {
        if reset.get() {
            enable_transition.set(false);

            if !disabled.get_untracked() {
                reset_bar_animation
                .set("animation: circular-progress-bar-fade-out 0.5s ease-out; animation-fill-mode: both;");
                reset_icon_animation.set(
                    "animation: circular-progress-bar-glow 0.5s ease; animation-fill-mode: both;",
                );

                // Trick to reset animation by removing it when ended
                set_timeout(
                    move || {
                        reset_bar_animation.set("opacity: 0;");
                        reset_icon_animation.set("");
                    },
                    std::time::Duration::from_millis(500),
                );
            }
        } else {
            enable_transition.set(true);
        }
    });

    view! {
        <div class="circular-progress-bar">
            <div
                class="relative w-full h-full aspect-square rounded-full overflow-hidden
                bg-radial from-stone-800 to-zinc-950 to-70%"
                style="contain: strict;"
            >

                <div
                    class="absolute rounded-full border border-[#120d0a] shadow-[inset_0_2px_3px_rgba(255,255,255,0.04),inset_0_-10px_16px_rgba(0,0,0,0.82)]"
                    style="inset: 18%; background:
                    radial-gradient(circle at 50% 46%, rgba(92,92,108,0.42) 0%, rgba(38,38,46,0.26) 24%, rgba(10,10,14,0.92) 58%, rgba(0,0,0,1) 82%),
                    linear-gradient(180deg, rgba(0,0,0,1), rgba(0,0,0,1));
                    radial-gradient(circle at 50% 46%,
                    rgba(108,108,128,0.52) 0%,
                    rgba(56,56,68,0.38) 22%,
                    rgba(18,18,24,0.82) 52%,
                    rgba(0,0,0,0.98) 78%,
                    rgba(0,0,0,1) 100%);"
                ></div>

                <div
                    class="absolute inset-0 rounded-full pointer-events-none"
                    style=format!(
                        "box-shadow:
                            inset 0 0 0 2px {BRASS_DARK},
                            inset 0 1px 0 2px rgba(255,233,185,0.38),
                            inset 0 -2px 0 2px rgba(60,40,16,0.92);",
                    )
                ></div>

                <div class="absolute inset-[5%] rounded-full">
                    {(0..SEGMENT_COUNT)
                        .map(|index| {
                            let angle = index as f64 * (360.0 / SEGMENT_COUNT as f64);
                            view! {
                                <div
                                    class="absolute inset-0 flex items-start justify-center"
                                    style=format!("transform: rotate({angle}deg);")
                                >
                                    <div
                                        class=move || {
                                            if enable_transition.get() {
                                                "relative mt-[1.2%] h-[13%] w-[22%] origin-center transition-[transform,opacity,filter] duration-300 ease-out"
                                            } else {
                                                "relative mt-[1.2%] h-[13%] w-[22%] origin-center"
                                            }
                                        }
                                        class:brightness-60=move || disabled.get()
                                        style=move || {
                                            let is_active = index < active_segments.get();
                                            let (background, border, shadow, opacity, scale) = if is_active {
                                                (
                                                    format!(
                                                        "linear-gradient(135deg,
                                                            rgba(255,248,230,0.92) 0%,
                                                            rgba(255,226,178,0.88) 12%,
                                                            rgba(255,194,108,0.78) 20%,
                                                            {SEGMENT_BRIGHT} 34%,
                                                            {bar_color} 54%,
                                                            {SEGMENT_MID} 76%,
                                                            {SEGMENT_DARK} 100%),
                                                         radial-gradient(circle at 26% 18%,
                                                            rgba(255,248,231,0.46) 0%,
                                                            rgba(255,220,168,0.22) 18%,
                                                            transparent 42%),
                                                         linear-gradient(315deg,
                                                            rgba(255,170,64,0.18) 0%,
                                                            transparent 40%)",
                                                    ),
                                                    "rgba(255,219,170,0.82)".to_string(),
                                                    format!(
                                                        "-2px -2px 6px rgba(255,242,214,0.10),
                                                         0 0 10px {SEGMENT_GLOW},
                                                         0 0 20px {bar_color},
                                                         inset 2px 2px 3px rgba(255,244,220,0.34),
                                                         inset -4px -5px 8px rgba(58,18,6,0.52)",
                                                    ),
                                                    "1".to_string(),
                                                    "scale(1.00)".to_string(),
                                                )
                                            } else {
                                                (
                                                    format!(
                                                        "linear-gradient(135deg,
                                                            rgba(255,218,164,0.14) 0%,
                                                            rgba(152,70,22,0.20) 18%,
                                                            rgba(72,24,8,0.88) 66%,
                                                            rgba(14,8,8,0.98) 100%),
                                                         radial-gradient(circle at 28% 20%,
                                                            rgba(255,226,180,0.12) 0%,
                                                            transparent 38%)",
                                                    )
                                                        .to_string(),
                                                    "rgba(255,176,108,0.18)".to_string(),
                                                    "inset 1px 1px 2px rgba(255,241,219,0.10),
                                                     inset -3px -5px 8px rgba(0,0,0,0.54),
                                                     0 0 0 1px rgba(0,0,0,0.30)"
                                                        .to_string(),
                                                    "0.74".to_string(),
                                                    "scale(0.97)".to_string(),
                                                )
                                            };
                                            format!(
                                                "clip-path: polygon(18% 0%, 82% 0%, 72% 100%, 28% 100%);
                                                 border-radius: 4px 4px 10px 10px;
                                                 background: {background};
                                                 border: 1px solid {border};
                                                 box-shadow: {shadow};
                                                 opacity: {opacity};
                                                 transform: {scale};",
                                            )
                                        }
                                    >
                                        <div
                                            class="absolute left-[18%] top-[10%] h-[18%] w-[42%]"
                                            style="clip-path: polygon(0% 100%, 64% 0%, 100% 14%, 34% 100%); background: linear-gradient(135deg, rgba(255,252,246,0.96), rgba(255,245,224,0.22) 48%, rgba(255,244,222,0.02));"
                                        ></div>
                                        <div
                                            class="absolute inset-x-[24%] top-[20%] h-[50%]"
                                            style=move || {
                                                if index < active_segments.get() {
                                                    "clip-path: polygon(16% 0%, 84% 0%, 74% 100%, 26% 100%);
                                                     background: linear-gradient(135deg, rgba(255,231,194,0.18), rgba(255,145,30,0.03) 46%, rgba(32,10,6,0.00));"
                                                        .to_string()
                                                } else {
                                                    "clip-path: polygon(16% 0%, 84% 0%, 74% 100%, 26% 100%);
                                                     background: linear-gradient(135deg, rgba(255,214,170,0.05), rgba(255,120,28,0.01) 46%, rgba(32,10,6,0.00));"
                                                        .to_string()
                                                }
                                            }
                                        ></div>
                                        <div
                                            class="absolute inset-x-[10%] bottom-[4%] h-[24%] rounded-full blur-[3px]"
                                            style=move || {
                                                if index < active_segments.get() {
                                                    format!(
                                                        "background: radial-gradient(circle, rgba(255,224,172,0.78) 0%, {SEGMENT_BRIGHT} 36%, {bar_color} 68%, transparent 100%);
                                                         opacity: 0.68;",
                                                    )
                                                } else {
                                                    "background: radial-gradient(circle, rgba(255,150,68,0.10) 0%, rgba(76,29,9,0.04) 75%, transparent 100%);
                                                     opacity: 0.14;"
                                                        .to_string()
                                                }
                                            }
                                        ></div>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>

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
