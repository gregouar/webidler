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
                class="relative w-full h-full aspect-square rounded-full overflow-hidden"
                style="contain: strict; background:
                radial-gradient(circle at 30% 25%, rgba(255,255,255,0.05), transparent 20%),
                linear-gradient(180deg, rgba(24,17,15,0.98), rgba(6,6,8,1));"
            >
                <div
                    class="absolute inset-0 rounded-full"
                    style=format!(
                        "background:
                            radial-gradient(circle, transparent 0 90%, rgba(0,0,0,0) 100%),
                            linear-gradient(180deg, rgba(0,0,0,1) 0%, rgba(0,0,0,1) 100%);
                         box-shadow:
                            inset 0 1px 2px rgba(255,255,255,0.03),
                            inset 0 -8px 10px rgba(0,0,0,0.82),
                            0 0 0 1px rgba(16,10,7,0.88);",
                    )
                ></div>

                <div
                    class="absolute inset-[4%] rounded-full"
                    style=format!(
                        "background:
                            radial-gradient(circle, transparent 0 90%, rgba(0,0,0,0) 100%),
                            linear-gradient(180deg, rgba(0,0,0,1) 0%, rgba(0,0,0,1) 100%);
                         box-shadow:
                            inset 0 1px 2px rgba(255,255,255,0.02),
                            inset 0 -4px 8px rgba(0,0,0,0.72);",
                    )
                ></div>

                <div
                    class="absolute inset-0 rounded-full pointer-events-none"
                    style=format!(
                        "box-shadow:
                            inset 0 0 0 2px {BRASS_DARK},
                            inset 0 1px 0 2px rgba(255,233,185,0.38),
                            inset 0 -2px 0 2px rgba(60,40,16,0.92);"
                    )
                ></div>

                <div
                    class="absolute inset-[5%] rounded-full"
                    style="background:
                    radial-gradient(circle, transparent 0 79%, rgba(0,0,0,1) 82%, rgba(0,0,0,1) 86%, transparent 89%);"
                >
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
                                                        "linear-gradient(180deg,
                                                            rgba(255,247,230,0.96) 0%,
                                                            rgba(255,214,150,0.90) 16%,
                                                            {SEGMENT_BRIGHT} 34%,
                                                            {bar_color} 50%,
                                                            {SEGMENT_MID} 74%,
                                                            {SEGMENT_DARK} 100%)",
                                                    ),
                                                    SEGMENT_EDGE.to_string(),
                                                    format!(
                                                        "0 0 12px {SEGMENT_GLOW},
                                                         0 0 24px {bar_color},
                                                         inset 0 1px 1px rgba(255,246,228,0.70),
                                                         inset 0 -7px 9px rgba(74,24,6,0.52)",
                                                    ),
                                                    "1".to_string(),
                                                    "scale(1.00)".to_string(),
                                                )
                                            } else {
                                                (
                                                    format!(
                                                        "linear-gradient(180deg,
                                                            rgba(255,214,166,0.14) 0%,
                                                            rgba(118,50,16,0.20) 18%,
                                                            rgba(60,20,7,0.92) 62%,
                                                            rgba(14,8,8,0.98) 100%)",
                                                    )
                                                        .to_string(),
                                                    "rgba(255,173,102,0.20)".to_string(),
                                                    "inset 0 1px 1px rgba(255,241,219,0.10),
                                                     inset 0 -6px 9px rgba(0,0,0,0.54),
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
                                            class="absolute inset-x-[18%] top-[8%] h-[24%]"
                                            style="clip-path: polygon(10% 100%, 50% 0%, 90% 100%); background: linear-gradient(180deg, rgba(255,252,246,0.92), rgba(255,244,222,0.10));"
                                        ></div>
                                        <div
                                            class="absolute inset-x-[24%] top-[20%] h-[50%]"
                                            style=move || {
                                                if index < active_segments.get() {
                                                    "clip-path: polygon(16% 0%, 84% 0%, 74% 100%, 26% 100%);
                                                     background: linear-gradient(180deg, rgba(255,219,170,0.36), rgba(255,126,24,0.04));"
                                                        .to_string()
                                                } else {
                                                    "clip-path: polygon(16% 0%, 84% 0%, 74% 100%, 26% 100%);
                                                     background: linear-gradient(180deg, rgba(255,214,170,0.06), rgba(255,120,28,0.01));"
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
                                        <div
                                            class="absolute inset-0"
                                            style=format!(
                                                "clip-path: polygon(18% 0%, 82% 0%, 72% 100%, 28% 100%);
                                                 border-left: 1px solid {SEGMENT_LEAD};
                                                 border-right: 1px solid {SEGMENT_LEAD};
                                                 border-top: 1px solid rgba(255,235,205,0.12);
                                                 border-bottom: 1px solid {SEGMENT_LEAD};
                                                 opacity: 0.92;",
                                            )
                                        ></div>
                                    </div>
                                </div>
                            }
                        })
                        .collect_view()}
                </div>

                <div
                    class="absolute inset-0 rounded-full will-change-opacity"
                    style=move || {
                        format!(
                            "background:
                                radial-gradient(circle, transparent 0 78%, rgba(255,224,182,0.04) 82%, {SEGMENT_BRIGHT} 86%, transparent 90%);
                                {};
                            ",
                            reset_bar_animation.get(),
                        )
                    }
                ></div>

                <div
                    class="absolute rounded-full border border-[#120d0a] shadow-[inset_0_2px_3px_rgba(255,255,255,0.04),inset_0_-10px_16px_rgba(0,0,0,0.82)]"
                    style="inset: 18%; background: linear-gradient(180deg, rgba(0,0,0,1), rgba(0,0,0,1));"
                ></div>

                <div
                    class="absolute top-1/2 start-1/2 transform -translate-y-1/2 -translate-x-1/2
                    scale-125
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
