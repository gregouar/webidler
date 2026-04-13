use leptos::{html::*, prelude::*};
use leptos_use::use_interval_fn;

use crate::components::settings::{GraphicsQuality, SettingsContext};

#[derive(Clone, Copy)]
pub struct CooldownClock(RwSignal<u64>);

pub fn provide_cooldown_clock() {
    let tick = RwSignal::new(0u64);
    use_interval_fn(
        move || {
            tick.update(|value| *value = value.wrapping_add(1));
        },
        200,
    );
    provide_context(CooldownClock(tick));
}

#[component]
pub fn HorizontalProgressBar(
    /// Percent value, must be between 0 and 100.
    #[prop(into)]
    value: Signal<f32>,
    /// Bar color, must be of format "bg-XXXX-NNN"
    bar_color: &'static str,
    /// Text
    #[prop(optional)]
    children: Option<Children>,
    // Instant reset
    #[prop(into,default = RwSignal::new(false))] reset: RwSignal<bool>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let settings: SettingsContext = expect_context();
    let set_value = move || {
        if reset.get() {
            0.0
        } else {
            value.get().clamp(0.0, 100.0).round() * 0.01
        }
    };

    let transition = move || {
        if reset.get() {
            "transition-none"
        } else {
            "transition-transform ease-linear duration-250 "
        }
    };

    // Trick to reset animation by removing it when ended
    let reset_bar_animation = RwSignal::new("opacity: 0;");
    Effect::new(move |_| {
        if reset.get() {
            reset_bar_animation
                .set("animation: horizontal-progress-bar-fade-out 0.5s ease-out; animation-fill-mode: both;");
            set_timeout(
                move || {
                    reset.set(false);
                },
                std::time::Duration::from_millis(100),
            );
            set_timeout(
                move || {
                    reset_bar_animation.set("opacity: 0;");
                },
                std::time::Duration::from_millis(500),
            );
        }
    });

    view! {
        <div
            class=move || {
                format!(
                    "relative flex w-full rounded-[4px] xl:rounded-[6px] {} {}",
                    match settings.graphics_quality() {
                        GraphicsQuality::High => {
                            "border border-[#6c5329] shadow-[0_3px_8px_rgba(0,0,0,0.42)]"
                        }
                        GraphicsQuality::Medium => "border border-[#6c5329] shadow-md",
                        GraphicsQuality::Low => "border border-[#5c4a2e]",
                    },
                    class.unwrap_or_default(),
                )
            }
            style:background-image="linear-gradient(180deg, rgba(214,177,102,0.08), rgba(0,0,0,0.14)), linear-gradient(180deg, rgba(35,33,39,0.96), rgba(17,16,20,1))"
        >
            {move || match settings.graphics_quality() {
                GraphicsQuality::High => {
                    view! {
                        <div class="pointer-events-none absolute inset-[1px] rounded-[3px] xl:rounded-[5px] border border-[#d5b16d]/18"></div>
                        <div class="pointer-events-none absolute inset-x-3 top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/45 to-transparent"></div>
                        <div class="pointer-events-none absolute inset-[3px] xl:inset-[4px] rounded-[2px] xl:rounded-[4px]
                        border border-black/45 bg-[linear-gradient(180deg,rgba(10,10,12,0.78),rgba(28,26,32,0.92))]"></div>
                    }
                        .into_any()
                }
                GraphicsQuality::Medium => {
                    view! {
                        <div class="pointer-events-none absolute inset-[1px] rounded-[3px] xl:rounded-[5px] border border-[#d5b16d]/18"></div>
                        <div class="pointer-events-none absolute inset-[3px] xl:inset-[4px] rounded-[2px] xl:rounded-[4px] border border-black/45  bg-[linear-gradient(180deg,rgba(14,14,17,0.8),rgba(28,26,31,0.92))]"></div>
                    }
                        .into_any()
                }
                GraphicsQuality::Low => {
                    view! {
                        <div class="pointer-events-none absolute inset-[3px] xl:inset-[4px] rounded-[2px] xl:rounded-[4px] border border-black/45 bg-[linear-gradient(180deg,rgba(14,14,17,0.8),rgba(28,26,31,0.92))]"></div>
                    }
                        .into_any()
                }
            }}
            <div class="w-full rounded-[4px] xl:rounded-[6px] p-[3px] xl:p-[4px]">
                <div
                    class=move || {
                        format!(
                            "relative block h-full w-full origin-left
                            {} {} {} {}",
                            if settings.uses_heavy_effects() {
                                "shadow-[inset_0_1px_0_rgba(255,255,255,0.18),inset_0_-1px_0_rgba(0,0,0,0.2)]"
                            } else {
                                ""
                            },
                            if settings.uses_heavy_effects() {
                                "before:absolute before:inset-0 before:bg-[linear-gradient(90deg,rgba(255,255,255,0.16),transparent_22%,transparent_78%,rgba(0,0,0,0.12))]"
                            } else {
                                ""
                            },
                            bar_color,
                            transition(),
                        )
                    }
                    style=move || format!("transform: scaleX({});", set_value())
                ></div>
                // Fake copy for glow effect on reset
                <div
                    class=format!("absolute  h-full w-full inset-0 z-1 {}", bar_color)
                    style=reset_bar_animation
                ></div>
            </div>

            <div class="absolute inset-0 z-1 flex items-center justify-center text-white text-xs xl:text-sm pointer-events-none text-shadow shadow-black/90">
                {children.map(|children| children())}
            </div>
        </div>
    }
}

#[component]
pub fn VerticalProgressBar(
    // Percent value, must be between 0 and 100.
    #[prop(into)] value: Signal<f64>,
    // Bar color, must be of format "bg-XXXX-NNN"
    bar_color: &'static str,
    // Instant reset
    #[prop(into,default = Signal::derive(|| false))] reset: Signal<bool>,
    #[prop(optional)] class: Option<&'static str>,
    #[prop(optional)] children: Option<Children>,
) -> impl IntoView {
    let settings: SettingsContext = expect_context();
    let set_value = move || {
        if reset.get() { 0.0 } else { value.get() }
    };

    // Trick to reset animation by removing it when ended
    let reset_bar_animation = RwSignal::new("opacity: 0;");
    Effect::new(move |_| {
        if reset.get() {
            reset_bar_animation
                .set("animation: vertical-progress-bar-fade-out 0.5s ease-out; animation-fill-mode: both;");
            set_timeout(
                move || {
                    reset_bar_animation.set("opacity: 0;");
                },
                std::time::Duration::from_millis(500),
            );
        }
    });

    view! {
        <div
            class=move || {
                format!(
                    "relative flex flex-col justify-end h-full rounded-[4px] xl:rounded-[6px] {} {}",
                    match settings.graphics_quality() {
                        GraphicsQuality::High => {
                            "border border-[#6c5329] shadow-[0_3px_8px_rgba(0,0,0,0.42)]"
                        }
                        GraphicsQuality::Medium => "border border-[#6c5329] shadow-md",
                        GraphicsQuality::Low => "border border-[#5c4a2e]",
                    },
                    class.unwrap_or_default(),
                )
            }
            style:background-image="linear-gradient(180deg, rgba(214,177,102,0.08), rgba(0,0,0,0.14)), linear-gradient(180deg, rgba(35,33,39,0.96), rgba(17,16,20,1))"
        >
            {move || match settings.graphics_quality() {
                GraphicsQuality::High => {
                    view! {
                        <div class="pointer-events-none absolute inset-[1px] rounded-[3px] xl:rounded-[5px] border border-[#d5b16d]/18"></div>
                        <div class="pointer-events-none absolute inset-x-[2px] top-[1px] h-px bg-gradient-to-r from-transparent via-[#edd39a]/45 to-transparent"></div>
                        <div class="pointer-events-none absolute inset-[3px] xl:inset-[4px] rounded-[2px] xl:rounded-[4px] border border-black/45 bg-[linear-gradient(180deg,rgba(10,10,12,0.82),rgba(28,26,32,0.92))]"></div>
                    }
                        .into_any()
                }
                GraphicsQuality::Medium => {

                    view! {
                        <div class="pointer-events-none absolute inset-[1px] rounded-[3px] xl:rounded-[5px] border border-[#d5b16d]/18"></div>
                        <div class="pointer-events-none absolute inset-[3px] xl:inset-[4px] rounded-[2px] xl:rounded-[4px] border border-black/45 bg-[linear-gradient(180deg,rgba(14,14,17,0.8),rgba(28,26,31,0.92))]"></div>
                    }
                        .into_any()
                }
                GraphicsQuality::Low => {

                    view! {
                        <div class="pointer-events-none absolute inset-[3px] xl:inset-[4px] rounded-[2px] xl:rounded-[4px] border border-black/45 bg-[linear-gradient(180deg,rgba(14,14,17,0.8),rgba(28,26,31,0.92))]"></div>
                    }
                        .into_any()
                }
            }}
            <div class="h-full rounded-[4px] xl:rounded-[6px] p-[3px] xl:p-[4px]">
                <div
                    class=move || {
                        format!(
                            "relative block h-full w-full origin-bottom
                            transition-progress-bar
                            {} {} {}",
                            if settings.uses_heavy_effects() {
                                "shadow-[inset_0_1px_0_rgba(255,255,255,0.18),inset_0_-1px_0_rgba(0,0,0,0.2)]"
                            } else {
                                ""
                            },
                            if settings.uses_heavy_effects() {
                                "before:absolute before:inset-0 before:bg-[linear-gradient(180deg,rgba(255,255,255,0.16),transparent_20%,transparent_80%,rgba(0,0,0,0.12))]"
                            } else {
                                ""
                            },
                            bar_color,
                        )
                    }
                    class:transition-progress-bar=move || !reset.get()
                    style=move || format!("transform: scaleY({});", set_value())
                ></div>

                // Fake copy for glow effect on reset
                <div
                    class=format!("absolute inset-0 z-1 h-full w-full {}", bar_color)
                    style=reset_bar_animation
                ></div>
            </div>
            <div class="absolute inset-0 z-1 flex items-center justify-center text-white text-xs xl:text-sm rounded-[4px] xl:rounded-[6px] text-shadow shadow-black/90">
                {children.map(|children| children())}
            </div>
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
    #[prop(optional)] tint_background: Option<&'static str>,
    #[prop(optional)] class: Option<&'static str>,
    // Inside the circular bar
    children: Children,
) -> impl IntoView {
    let settings: SettingsContext = expect_context();

    let reset_icon_animation = RwSignal::new("");
    let active_buffer = RwSignal::new(false);
    let front_progress = RwSignal::new(value.get_untracked().clamp(0.0, 1.0) * 100.0);
    let back_progress = RwSignal::new(0.0);
    let last_reset = RwSignal::new(reset.get_untracked());
    let front_right_transition_ms = RwSignal::new(200u16);
    let front_left_transition_ms = RwSignal::new(200u16);
    let front_left_delay_ms = RwSignal::new(0u16);
    let back_right_transition_ms = RwSignal::new(200u16);
    let back_left_transition_ms = RwSignal::new(200u16);
    let back_left_delay_ms = RwSignal::new(0u16);

    let set_half_transition_timing =
        |previous: f64,
         next: f64,
         right_transition: RwSignal<u16>,
         left_transition: RwSignal<u16>,
         left_delay: RwSignal<u16>| {
            const BASE_MS: f64 = 200.0;

            if next > previous && previous < 50.0 && next > 50.0 {
                let handoff_ratio = ((50.0 - previous) / (next - previous)).clamp(0.0, 1.0);
                let right_ms = (BASE_MS * handoff_ratio).round().clamp(1.0, BASE_MS) as u16;
                let left_ms = (BASE_MS - f64::from(right_ms)).round().clamp(1.0, BASE_MS) as u16;
                right_transition.set(right_ms);
                left_transition.set(left_ms);
                left_delay.set(right_ms);
            } else {
                right_transition.set(200);
                left_transition.set(200);
                left_delay.set(0);
            }
        };

    Effect::new(move |_| {
        let is_reset = reset.get();
        let was_reset = last_reset.get_untracked();
        let progress = value.get().clamp(0.0, 1.0) * 100.0;

        if is_reset && !was_reset {
            front_right_transition_ms.set(200);
            front_left_transition_ms.set(200);
            front_left_delay_ms.set(0);
            back_right_transition_ms.set(200);
            back_left_transition_ms.set(200);
            back_left_delay_ms.set(0);

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
        } else if !is_reset {
            if active_buffer.get() {
                let previous = back_progress.get_untracked();
                set_half_transition_timing(
                    previous,
                    progress,
                    back_right_transition_ms,
                    back_left_transition_ms,
                    back_left_delay_ms,
                );
                back_progress.set(progress);
            } else {
                let previous = front_progress.get_untracked();
                set_half_transition_timing(
                    previous,
                    progress,
                    front_right_transition_ms,
                    front_left_transition_ms,
                    front_left_delay_ms,
                );
                front_progress.set(progress);
            }
        }

        last_reset.set(is_reset);
    });

    let front_right_deg = move || front_progress.get().clamp(0.0, 50.0) * 3.6 - 180.0;
    let front_left_deg = move || (front_progress.get() - 50.0).clamp(0.0, 50.0) * 3.6 - 180.0;
    let back_right_deg = move || back_progress.get().clamp(0.0, 50.0) * 3.6 - 180.0;
    let back_left_deg = move || (back_progress.get() - 50.0).clamp(0.0, 50.0) * 3.6 - 180.0;
    // const RING_INSET_DEPTH: &str =
    //     "radial-gradient(circle at 50% 50%, rgba(0,0,0,0.0) 60%, rgba(0,0,0,0.5) 73%),";
    const RING_INSET_DEPTH: &str = "";

    view! {
        <div class="circular-progress-bar">
            <div class=move || {
                format!(
                    "relative w-full h-full aspect-square rounded-full {} {}",
                    match settings.graphics_quality() {
                        GraphicsQuality::High => {
                            "border border-[#6c5329] bg-stone-900 shadow-[0_0_15px_rgba(0,0,0,0.95),inset_0_1px_0_rgba(230,208,154,0.22),inset_0_-1px_0_rgba(0,0,0,0.45),inset_0_0_10px_rgba(0,0,0,0.95)]"
                        }
                        GraphicsQuality::Medium => {
                            "border border-[#6c5329] bg-stone-900 shadow-[0_0_15px_rgba(0,0,0,0.95)]"
                        }
                        GraphicsQuality::Low => "border border-[#5c4a2e] bg-zinc-900",
                    },
                    class.unwrap_or_default(),
                )
            }>
                {move || match settings.graphics_quality() {
                    GraphicsQuality::High | GraphicsQuality::Medium => {
                        view! {
                            <div class="pointer-events-none absolute inset-[1px] rounded-full border border-[#d5b16d]/18"></div>
                        }
                            .into_any()
                    }
                    GraphicsQuality::Low => {
                        let _: () = view! { <></> };
                        ().into_any()
                    }
                }}
                <div
                    class="absolute inset-0 transition-opacity duration-500"
                    class:opacity-0=move || disabled.get() || active_buffer.get()
                >
                    <div
                        class="absolute inset-y-0 right-0 w-[calc(50%+1px)] overflow-clip"
                        class:invisible=move || front_progress.get() <= 0.0
                    >
                        <div
                            class="absolute inset-y-0 -left-full w-[200%] rounded-full transform-gpu will-change-transform"
                            style=move || {
                                let background = if matches!(
                                    settings.graphics_quality(),
                                    GraphicsQuality::Low
                                ) {
                                    format!(
                                        "linear-gradient(90deg, transparent 50%, {} 50%)",
                                        bar_color,
                                    )
                                } else {
                                    format!(
                                        "{} linear-gradient(90deg, transparent 50%, {} 50%)",
                                        RING_INSET_DEPTH,
                                        bar_color,
                                    )
                                };
                                format!(
                                    "background: {}; /*mask-image: linear-gradient(90deg, transparent 49%, #000 49.2%); -webkit-mask-image: linear-gradient(90deg, transparent 49%, #000 49.2%);*/ transform: rotate({}deg); transform-origin: 50% 50%; transition: transform {}ms linear;",
                                    background,
                                    front_right_deg(),
                                    front_right_transition_ms.get(),
                                )
                            }
                        ></div>
                    </div>
                    <div
                        class="absolute inset-y-0 left-0 w-1/2 overflow-clip"
                        class:invisible=move || front_progress.get() <= 50.0
                    >
                        <div
                            class="absolute inset-y-0 left-0 w-[200%] rounded-full transform-gpu will-change-transform"
                            style=move || {
                                let background = if matches!(
                                    settings.graphics_quality(),
                                    GraphicsQuality::Low
                                ) {
                                    format!(
                                        "linear-gradient(90deg, {} 50%, transparent 50%)",
                                        bar_color,
                                    )
                                } else {
                                    format!(
                                        "{} linear-gradient(90deg, {} 50%, transparent 50%)",
                                        RING_INSET_DEPTH,
                                        bar_color,
                                    )
                                };
                                format!(
                                    "background: {}; /*mask-image: linear-gradient(90deg, #000 50.8%, transparent 51%); -webkit-mask-image: linear-gradient(90deg, #000 50.8%, transparent 51%);*/ transform: rotate({}deg); transform-origin: 50% 50%; transition: transform {}ms linear {}ms;",
                                    background,
                                    front_left_deg(),
                                    front_left_transition_ms.get(),
                                    front_left_delay_ms.get(),
                                )
                            }
                        ></div>
                    </div>
                </div>
                <div
                    class="absolute inset-0 transition-opacity duration-500"
                    class:opacity-0=move || disabled.get() || !active_buffer.get()
                >
                    <div
                        class="absolute inset-y-0 right-0 w-[calc(50%+1px)] overflow-clip"
                        class:invisible=move || back_progress.get() <= 0.0
                    >
                        <div
                            class="absolute inset-y-0 -left-full w-[200%] rounded-full transform-gpu will-change-transform"
                            style=move || {
                                let background = if matches!(
                                    settings.graphics_quality(),
                                    GraphicsQuality::Low
                                ) {
                                    format!(
                                        "linear-gradient(90deg, transparent 50%, {} 50%)",
                                        bar_color,
                                    )
                                } else {
                                    format!(
                                        "{} linear-gradient(90deg, transparent 50%, {} 50%)",
                                        RING_INSET_DEPTH,
                                        bar_color,
                                    )
                                };
                                format!(
                                    "background: {}; /*mask-image: linear-gradient(90deg, transparent 49%, #000 49.2%); -webkit-mask-image: linear-gradient(90deg, transparent 49%, #000 49.2%);*/ transform: rotate({}deg); transform-origin: 50% 50%; transition: transform {}ms linear;",
                                    background,
                                    back_right_deg(),
                                    back_right_transition_ms.get(),
                                )
                            }
                        ></div>
                    </div>
                    <div
                        class="absolute inset-y-0 left-0 w-1/2 overflow-clip"
                        class:invisible=move || back_progress.get() <= 50.0
                    >
                        <div
                            class="absolute inset-y-0 left-0 w-[200%] rounded-full transform-gpu will-change-transform"
                            style=move || {
                                let background = if matches!(
                                    settings.graphics_quality(),
                                    GraphicsQuality::Low
                                ) {
                                    format!(
                                        "linear-gradient(90deg, {} 50%, transparent 50%)",
                                        bar_color,
                                    )
                                } else {
                                    format!(
                                        "{} linear-gradient(90deg, {} 50%, transparent 50%)",
                                        RING_INSET_DEPTH,
                                        bar_color,
                                    )
                                };
                                format!(
                                    "background: {}; /*mask-image: linear-gradient(90deg, #000 50.8%, transparent 51%); -webkit-mask-image: linear-gradient(90deg, #000 50.8%, transparent 51%);*/ transform: rotate({}deg); transform-origin: 50% 50%; transition: transform {}ms linear {}ms;",
                                    background,
                                    back_left_deg(),
                                    back_left_transition_ms.get(),
                                    back_left_delay_ms.get(),
                                )
                            }
                        ></div>
                    </div>
                </div>
                <div class=move || {
                    match settings.graphics_quality() {
                        GraphicsQuality::High => {
                            format!(
                                "absolute inset-{} xl:inset-{bar_width} rounded-full
                            bg-radial {} to-zinc-950 to-70%
                            border border-[#6d532e]/70 shadow-[inset_0_2px_6px_rgba(0,0,0,0.55),inset_0_1px_0_rgba(236,210,148,0.14),0_1px_2px_rgba(0,0,0,0.35)]",
                                bar_width / 2,
                                tint_background.unwrap_or("from-stone-600"),
                            )
                        }
                        GraphicsQuality::Medium => {
                            format!(
                                "absolute inset-{} xl:inset-{bar_width} rounded-full
                            bg-radial {} to-zinc-950 to-70%
                            border border-[#6d532e]/70",
                                bar_width / 2,
                                tint_background.unwrap_or("from-stone-600"),
                            )
                        }
                        GraphicsQuality::Low => {
                            format!(
                                "absolute inset-{} xl:inset-{bar_width} rounded-full
                            bg-radial {} to-zinc-950 to-70%
                            border border-[#5c4a2e]",
                                bar_width / 2,
                                tint_background.unwrap_or("from-stone-600"),
                            )
                        }
                    }
                }>// Icon
                </div>
                <div
                    class="absolute top-1/2 start-1/2 transform -translate-y-1/2 -translate-x-1/2
                    scale-120 xl:drop-shadow-[0_2px_0px_rgba(0,0,0,0.5)]
                    transition-transform duration-500"
                    style=reset_icon_animation
                    class:brightness-50=move || disabled.get()
                >
                    {children()}
                </div>
            </div>
        </div>
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
    let cooldown_clock = expect_context::<CooldownClock>();

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

    Effect::new(move |_| {
        cooldown_clock.0.get();
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
    });

    progress_value
}
