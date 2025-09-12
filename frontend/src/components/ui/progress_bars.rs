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
    #[prop(into,default = Signal::derive(|| false))] reset: Signal<bool>,
    #[prop(optional)] class: Option<&'static str>,
) -> impl IntoView {
    let set_value = move || {
        if reset.get() {
            0.0
        } else {
            value.get().clamp(0.0, 100.0).round()
        }
    };

    let transition = move || {
        if reset.get() {
            "transition-none"
        } else {
            "transition-all ease-linear duration-300 "
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
            @keyframes horizontal-progress-bar-fade-out {
                0% {
                    opacity: 1;
                    box-shadow: 0 0 0px rgba(59, 130, 246, 0);
                }
                50% {
                    box-shadow: 0 0 25px 10px oklch(92.4% 0.12 95.746);
                }
                100% {
                    opacity: 0;
                    box-shadow: 0 0 0px rgba(59, 130, 246, 0);
                }
            }
            "
        </style>
        <div class=format!(
            "
            relative flex w-full
            rounded-lg
            bg-stone-900 border border-neutral-950 
            shadow-md
            {}
            ",
            class.unwrap_or_default(),
        )>
            <div
                class=move || format!("flex flex-col {} rounded-lg {}", bar_color, transition())
                style:width=move || format!("{}%", set_value())
            >
                // Fake copy for glow effect on reset
                <div
                    class=format!("absolute inset-0 z-1 rounded-lg {}", bar_color)
                    style=reset_bar_animation
                ></div>
            </div>
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
) -> impl IntoView {
    view! {
        <div class="
            flex flex-col flex-nowrap justify-end h-full
            rounded-lg overflow-hidden
            bg-stone-900 border border-neutral-950
            shadow-md
            ">
            <div
                class={format!("{bar_color} rounded-lg overflow-hidden -all ease duration-300")}
                style:height=move || format!("{}%", value.get().clamp(0.0,100.0).round())
                style:-webkit-mask="linear-gradient(#fff 0 0)"
            ></div>
        </div>
    }
}

#[component]
pub fn CircularProgressBar(
    // Percent value, must be between 0 and 100.
    #[prop(into)] remaining_time: Signal<f32>,
    bar_color: &'static str,
    bar_width: u8,
    // Instant reset
    #[prop(into,default = Signal::derive(|| false))] reset: Signal<bool>,
    #[prop(into,default = Signal::derive(|| false))] disabled: Signal<bool>,
    // Inside the circular bar
    children: Children,
) -> impl IntoView {
    let progress_value = RwSignal::new(0.0f64);

    let reset_bar_animation = RwSignal::new("opacity: 0;");
    let reset_icon_animation = RwSignal::new("");
    let transition = RwSignal::new("transition: opacity 0.5s linear, --progress 0.250s linear;");

    Effect::new(move |_| {
        if reset.get() {
            progress_value.set(0.0);
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
        }
    });

    Effect::new(move |_| {
        if disabled.get() {
            set_timeout(
                move || {
                    progress_value.set(0.0);
                    transition.set("");
                },
                std::time::Duration::from_millis(500),
            );
        }
    });

    let rate = RwSignal::new(0.0);

    Effect::new(move || {
        let remaining_time = remaining_time.get() as f64;
        if remaining_time > 0.0 {
            rate.set((1.0 - progress_value.get_untracked()).clamp(0.0, 1.0) / remaining_time);
        }
    });

    use_interval_fn(
        move || {
            if !disabled.get() {
                transition.set("transition: opacity 0.5s linear, --progress 0.250s linear;");
                progress_value.update(|progress_value| {
                    *progress_value += rate.get_untracked() * 0.2;
                });
            }
        },
        200,
    );

    view! {
        <div class="circular-progress-bar">
            <style>
                "
                @keyframes circular-progress-bar-fade-out {
                 0% { opacity: 1; filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0)); }
                 50% { filter: drop-shadow(0 0 12px oklch(92.4% 0.12 95.746)); }
                 100% { opacity: 0; filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0)); }
                }
                @keyframes circular-progress-bar-glow {
                 0% { filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0));  }
                 50% { filter: drop-shadow(0 0 12px oklch(92.4% 0.12 95.746));  transform: scale(1.2); }
                 100% { filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0)); }
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
                      class="absolute inset-0 rounded-full"
                      class:opacity-0=move || disabled.get()
                        style=move || format!("
                            background: conic-gradient(
                                {bar_color} var(--progress),
                                transparent var(--progress) 100%
                            );
                            will-change: --progress;
                            {}
                        ",transition.get())
                        style:--progress=move || format!("{}%", progress_value.get() * 100.0)
                    ></div>

                    // For nice fade out during reset
                    <div
                        class="absolute inset-0 rounded-full"
                        style=move || format!("
                            background: {bar_color};
                            {}
                        ",reset_bar_animation.get())
                    ></div>

                    <div class=format!("absolute inset-{} xl:inset-{bar_width} rounded-full
                            bg-radial from-stone-600 to-zinc-950 to-70%", bar_width/2)></div>
                </div>

                <div
                    class="absolute top-1/2 start-1/2 transform -translate-y-1/2 -translate-x-1/2"
                    style=reset_icon_animation
                >
                    {children()}
                </div>
            </div>
        </div>
    }
}
