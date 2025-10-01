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
            value.get().clamp(0.0, 1.0)
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
                        format!(
                            "h-full origin-bottom will-change-transform {} {}",
                            bar_color,
                            transition(),
                        )
                    }
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
    // let progress_value = RwSignal::new(0.0f32);

    let reset_bar_animation = RwSignal::new("opacity: 0;");
    let reset_icon_animation = RwSignal::new("");
    let transition = RwSignal::new("transition: opacity 0.5s linear, --progress 0.250s linear;");

    Effect::new(move |_| {
        if reset.get() {
            // progress_value.set(0.0);
            transition.set("");

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
            transition.set("transition: opacity 0.5s linear, --progress 0.250s linear;");
        }
    });

    Effect::new(move |_| {
        if disabled.get() {
            set_timeout(
                move || {
                    // progress_value.set(0.0);
                    transition.set("");
                },
                std::time::Duration::from_millis(500),
            );
        }
    });

    // let rate = RwSignal::new(0.0);

    // Effect::new(move || {
    //     let remaining_time = remaining_time.get() as f64;
    //     if remaining_time > 0.0 {
    //         rate.set((1.0 - progress_value.get_untracked()).clamp(0.0, 1.0) / remaining_time);
    //     }
    // });

    // use_interval_fn(
    //     move || {
    //         if !disabled.get() {
    //             transition.set("transition: opacity 0.5s linear, --progress 0.250s linear;");
    //             progress_value.update(|progress_value| {
    //                 *progress_value += rate.get_untracked() * 0.2;
    //             });
    //         }
    //     },
    //     200,
    // );

    view! {
        <div class="circular-progress-bar">
            <style>
                "
                @keyframes circular-progress-bar-fade-out {
                 0% { opacity: 1; /*filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0));*/ }
                 /*50% { filter: drop-shadow(0 0 12px oklch(92.4% 0.12 95.746)); }*/
                 100% { opacity: 0; /*filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0));*/ }
                }
                @keyframes circular-progress-bar-glow {
                 /*0% { filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0)); }*/
                 50% { 
                  /*  filter: drop-shadow(0 0 12px oklch(92.4% 0.12 95.746));  */
                    transform: scale(1.2); 
                 }
                 /*100% { filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0)); }*/
                }

                @property --progress {
                    syntax: '<percentage>';
                    inherits: false;
                    initial-value: 0%;
                }
                "
            </style>
            <div class="relative">
                <div class="relative w-full h-full aspect-square rounded-full flex items-center justify-center bg-stone-900">
                    <div
                      class="absolute inset-0 rounded-full will-change-(--progress)"
                      class:opacity-0=move || disabled.get()
                        style=move || format!("
                            background: conic-gradient(
                                {bar_color} var(--progress),
                                transparent var(--progress) 100%
                            );
                            {}
                        ",transition.get())
                        style:--progress=move || format!("{}%", value.get() * 100.0)
                    ></div>

                    // For nice fade out during reset
                    <div
                        class="absolute inset-0 rounded-full will-change-[filter]"
                        style=move || format!("
                            background: {bar_color};
                            {}
                        ",reset_bar_animation.get())
                    ></div>

                    <div class=format!("absolute inset-{} xl:inset-{bar_width} rounded-full
                            bg-radial from-stone-600 to-zinc-950 to-70%", bar_width/2)></div>
                </div>

                <div
                    class="absolute top-1/2 start-1/2 transform -translate-y-1/2 -translate-x-1/2
                            will-change-[filter,transform] transition duration-500"
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
                    *progress_value += rate.get_untracked() * 0.2;
                });
            }
        },
        200,
    );

    progress_value
}
