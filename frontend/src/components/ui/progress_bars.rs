use leptos::{html::*, prelude::*};
use leptos_use::use_interval_fn;

#[component]
pub fn HorizontalProgressBar(
    /// Percent value, must be between 0 and 100.
    #[prop(into)]
    value: Signal<f32>,
    /// Bar color, must be of format "bg-XXXX-NNN"
    bar_color: &'static str,
    /// Text
    children: Children,
    // Instant reset
    #[prop(into,default = RwSignal::new(false))] reset: RwSignal<bool>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
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
        <style>
            "
            @keyframes horizontal-progress-bar-fade-out {
                0% {
                    opacity: 1;
                    filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0));  
                }
                50% {
                    filter: drop-shadow( 0 0 25px oklch(92.4% 0.12 95.746));  
                }
                100% {
                    opacity: 0;
                    filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0));  
                }
            }
            "
        </style>
        <div class=format!(
            "
            relative flex w-full
            rounded-lg
            bg-stone-900 border border-neutral-950 
            shadow-[inset_0_0_8px_rgba(0,0,0,0.4)]
            {}
            ",
            class.unwrap_or_default(),
        )>
            <div class="overflow-hidden w-full rounded-lg">
                <div
                    class=move || {
                        format!(
                            "h-full origin-left will-change-transform {} {}",
                            bar_color,
                            transition(),
                        )
                    }
                    style=move || format!("transform: scaleX({});", set_value())
                ></div>
            </div>

            // Fake copy for glow effect on reset
            <div
                class=format!("absolute inset-0 z-1 rounded-lg {}", bar_color)
                style=reset_bar_animation
            ></div>
            <div class="absolute inset-0 z-1 flex items-center justify-center text-white text-xs xl:text-sm pointer-events-none overflow-hidden">
                {children()}
            </div>
        </div>
    }
}

#[component]
pub fn VerticalProgressBar(
    // Percent value, must be between 0 and 100.
    #[prop(into)] value: Signal<f32>,
    // Bar color, must be of format "bg-XXXX-NNN"
    bar_color: &'static str,
    // Instant reset
    #[prop(into,default = Signal::derive(|| false))] reset: Signal<bool>,
) -> impl IntoView {
    let set_value = move || {
        if reset.get() {
            0.0
        } else {
            value.get()
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
                    reset_bar_animation.set("opacity: 0;");
                },
                std::time::Duration::from_millis(500),
            );
        }
    });

    view! {
        <style>
            "
            @keyframes vertical-progress-bar-fade-out {
                0% {
                    opacity: 1;
                    filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0));  
                }
                50% {
                    filter: drop-shadow( 0 0 25px oklch(92.4% 0.12 95.746));  
                }
                100% {
                    opacity: 0;
                    filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0));  
                }
            }
            "
        </style>

        <div class="
        relative flex flex-col justify-end h-full
        rounded-lg 
        bg-stone-900 border border-neutral-950 
        shadow-[inset_0_0_8px_rgba(0,0,0,0.4)]
        ">
            <div class="overflow-hidden h-full rounded-lg">
                <div
                    class=move || {
                        format!("h-full origin-bottom will-change-transform {}", bar_color)
                    }
                    class:transition-progress-bar=move || !reset.get()
                    style=move || format!("transform: scaleY({});", set_value())
                ></div>
            </div>
            // Fake copy for glow effect on reset
            <div
                class=format!("absolute rounded-lg inset-0 z-1 h-full {}", bar_color)
                style=reset_bar_animation
            ></div>
        </div>
    }
}

#[component]
pub fn CircularProgressBar(
    // Percent value, must be between 0 and 100.
    #[prop(into)] value: Signal<f32>,
    bar_color: &'static str,
    bar_width: u8,
    // Instant reset
    #[prop(into,default = Signal::derive(|| false))] reset: Signal<bool>,
    #[prop(into,default = Signal::derive(|| false))] disabled: Signal<bool>,
    // Inside the circular bar
    children: Children,
) -> impl IntoView {
    let reset_bar_animation = RwSignal::new("opacity: 0;");
    let reset_icon_animation = RwSignal::new("");

    let enable_transition = RwSignal::new(true);

    let in_transition = RwSignal::new(false);
    let next_value = RwSignal::new(0.0f32);

    let right_rotation = RwSignal::new(0.0);
    let bottom_rotation = RwSignal::new(0.0);
    let left_rotation = RwSignal::new(0.0);

    let hide_ul_overlay = RwSignal::new(false);
    let hide_bl_overlay = RwSignal::new(false);

    Effect::new(move |_| {
        if reset.get() {
            in_transition.set(false);

            enable_transition.set(false);
            right_rotation.set(0.0);
            bottom_rotation.set(0.0);
            left_rotation.set(0.0);

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

    // let node_ref: NodeRef<leptos::html::Div> = NodeRef::new();

    // // reactive states
    // let target = RwSignal::new(0.0);
    // let current = RwSignal::new(0.0);

    // // whenever `value` changes, update target
    // Effect::new(move |_| {
    //     target.set(value.get().clamp(0.0, 1.0));
    //     // if we're idle (current already == target), kick off a transition now
    //     if (current.get_untracked() - target.get_untracked()).abs() < 1e-6 {
    //         // set style immediately
    //         if let Some(el) = node_ref.get() {
    //             let percent = target.get_untracked() * 100.0;
    //             el.style("transform", &format!("scaleY({})", target.get_untracked()));
    //         }
    //     }
    // });

    // // attach transitionend handler
    // leptos::window_event_listener(leptos::ev::TransitionEnd, move |_| {
    //     let t = target.get_untracked();
    //     let c = current.get_untracked();
    //     if (t - c).abs() > 1e-6 {
    //         current.set(t);
    //         if let Some(el) = node_ref.get() {
    //             el.style("transform", &format!("scaleY({})", t));
    //         }
    //     }
    // });

    Effect::new(move |_| {
        if !in_transition.get() {
            next_value.set(value.get());
        }
    });

    Effect::new(move |_| {
        let value = next_value.get().clamp(0.0, 1.2);
        left_rotation.set(value * 360.0);

        if value <= 0.5 {
            right_rotation.set(value.clamp(0.0, 0.5) * 360.0);
            hide_bl_overlay.set(false);
        } else {
            right_rotation.set(180.0);
            hide_bl_overlay.set(true);
        }

        if value <= 0.75 {
            bottom_rotation.set(value.clamp(0.0, 0.75) * 360.0);
            hide_ul_overlay.set(false);
        } else {
            bottom_rotation.set(270.0);
            hide_ul_overlay.set(true);
        }
    });

    view! {
        <div class="circular-progress-bar">
            <style>
                "
                @keyframes circular-progress-bar-fade-out {
                 0% { opacity: 1; }
                 100% { opacity: 0; }
                }
                @keyframes circular-progress-bar-glow {
                 50% { 
                    transform: scale(1.2); 
                 }
                }
                "
            </style>
            <div class="relative">

                // Progress bar
                <div class="relative w-full aspect-square rounded-full overflow-hidden bg-stone-900">

                    // Second container to add ring to hide not pixel perfect overflow
                    <div
                        class="absolute inset-px rounded-full overflow-hidden"
                        class:transition-progress-bar=enable_transition
                        class:opacity-0=disabled
                    >
                        // Left half
                        <div
                            on:transitionstart=move |_| {
                                in_transition.set(true);
                            }
                            on:transitionend=move |_| {
                                next_value.set(value.get_untracked());
                                in_transition.set(false);
                            }
                            class="absolute inset-0 origin-right w-1/2 will-change-transform"
                            class:transition-progress-bar=enable_transition
                            style=move || {
                                format!(
                                    "transform: rotate({}deg); background: {bar_color};",
                                    left_rotation.get(),
                                )
                            }
                        ></div>

                        // Bottom half
                        <div
                            class="absolute left-0 bottom-0 right-0 origin-top
                            h-1/2 will-change-transform"
                            class:transition-progress-bar=enable_transition
                            style=move || {
                                format!(
                                    "transform: rotate({}deg); background: {bar_color};",
                                    bottom_rotation.get() + 90.0,
                                )
                            }
                        ></div>

                        // Right half
                        <div
                            class="absolute top-0 bottom-0 right-0 origin-left
                            w-1/2 will-change-transform"
                            class:transition-progress-bar=enable_transition
                            style=move || {
                                format!(
                                    "transform: rotate({}deg); background: {bar_color};",
                                    right_rotation.get() + 180.0,
                                )
                            }
                        ></div>
                    </div>

                    // Upper-left overlay
                    <div
                        class="absolute inset-0 bg-stone-900 origin-bottom-right size-1/2"
                        class:hidden=hide_ul_overlay
                    ></div>

                    // Bottom-left overlay
                    <div
                        class="absolute inset-0 bg-stone-900 origin-right w-1/2"
                        class:hidden=hide_bl_overlay
                    ></div>
                </div>

                // For nice fade out during reset
                <div
                    class="absolute inset-0 rounded-full will-change-opacity"
                    style=move || {
                        format!("background: {bar_color}; {}", reset_bar_animation.get())
                    }
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

pub fn predictive_cooldown(
    remaining_time: Signal<f32>,
    reset: Signal<bool>,
    disabled: Signal<bool>,
) -> RwSignal<f32> {
    let progress_value = RwSignal::new(0.0f32);
    let rate = RwSignal::new(0.0);

    Effect::new(move || {
        let remaining_time = remaining_time.get();
        if remaining_time > 0.0 {
            rate.set((1.0 - progress_value.get_untracked()).clamp(0.0, 1.0) / remaining_time);
        }
    });

    Effect::new(move || {
        if reset.get() {
            progress_value.set(0.0);
        }
    });

    use_interval_fn(
        move || {
            if !disabled.get_untracked() {
                progress_value.update(|progress_value| {
                    let rate = rate.get_untracked();
                    if *progress_value < 1.2 {
                        *progress_value += rate * 0.05;
                    }
                    if remaining_time.get_untracked() == 0.0 && rate == 0.0 {
                        *progress_value = 1.0;
                    }
                });
            }
        },
        50,
    );

    progress_value
}
