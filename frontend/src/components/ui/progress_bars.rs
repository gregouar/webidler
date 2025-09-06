use leptos::{html::*, prelude::*};

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
            <div class="absolute inset-0 z-1 flex items-center justify-center text-white text-sm pointer-events-none overflow-hidden">
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
                class={format!("{bar_color} rounded-lg overflow-hidden transition-all ease duration-300")}
                style:height=move || format!("{}%", value.get().clamp(0.0,100.0).round())
                style:-webkit-mask="linear-gradient(#fff 0 0)"
            ></div>
        </div>
    }
}

// #[component]
// pub fn CircularProgressBar(
//     // Percent value, must be between 0 and 100.
//     #[prop(into)] value: Signal<f32>,
//     // Bar color, must be of format "text-XXXX-NNN"
//     bar_color: &'static str,
//     // Width of the progress bar
//     #[prop(default = 2)] bar_width: u16,
//     // Instant reset
//     #[prop(into,default = Signal::derive(|| false))] reset: Signal<bool>,
//     // Inside the circular bar
//     children: Children,
// ) -> impl IntoView {
//     let bar_width = bar_width * 5;
//     let set_value = move || {
//         if reset.get() {
//             452.389
//         } else {
//             452.389 - value.get().clamp(0.0, 100.0) * 452.389 / 100.0
//         }
//     };

//     let transition = move || {
//         if reset.get() {
//             "transition-none"
//         } else {
//             "transition-all ease-linear duration-200 "
//         }
//     };
//     // Trick to reset animation by removing it when ended
//     let reset_bar_animation = RwSignal::new("opacity: 0;");
//     let reset_icon_animation = RwSignal::new("");
//     Effect::new(move |_| {
//         if reset.get() {
//             reset_bar_animation
//                 .set("animation: circular-progress-bar-fade-out 0.5s ease-out; animation-fill-mode: both;");
//             reset_icon_animation
//                 .set("animation: circular-progress-bar-glow 0.5s ease; animation-fill-mode: both;");
//             set_timeout(
//                 move || {
//                     reset_bar_animation.set("opacity: 0;");
//                     reset_icon_animation.set("");
//                 },
//                 std::time::Duration::from_millis(500),
//             );
//         }
//     });

//     view! {
//         <div class="circular-progress-bar">
//             <style>
//                 "
//                 @keyframes circular-progress-bar-fade-out {
//                  0% { opacity: 1; filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0)); }
//                  50% { filter: drop-shadow(0 0 12px oklch(92.4% 0.12 95.746)); }
//                  100% { opacity: 0; filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0)); }
//                 }
//                 @keyframes circular-progress-bar-glow {
//                  0% { filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0));  }
//                  50% { filter: drop-shadow(0 0 12px oklch(92.4% 0.12 95.746));  transform: translate(-50%, -50%) scale(1.2); }
//                  100% { filter: drop-shadow(0 0 0px rgba(59, 130, 246, 0)); }
//                 }
//                 "
//             </style>
//             <div class="relative">
//                 <svg class="size-full overflow-visible" viewBox="0 0 180 180">
//                     <defs>
//                         <clipPath id="ring" clip-rule="evenodd">
//                             <path d="M0-81A81 81 0 0 1 0 81A81 81 0 0 1 0-81z
//                             M0-63A63 63 0 0 1 0 63A63 63 0 0 1 0-63z" />
//                         </clipPath>
//                         <radialGradient id="inner-gradient" cx="50%" cy="50%" r="50%">
//                             <stop offset="0%" stop-color="oklch(44.4% 0.011 73.639)" />
//                             <stop offset="100%" stop-color="oklch(14.1% 0.005 285.823)" />
//                         </radialGradient>
//                     </defs>

//                     <g transform="translate(90,90)">
//                         <g clip-path="url(#ring)">
//                             <circle
//                                 class="stroke-current text-stone-900"
//                                 cx="0"
//                                 cy="2.5"
//                                 r="72"
//                                 stroke-width=bar_width
//                             />
//                         </g>
//                         <circle cx="0" cy="0" r="63" fill="url(#inner-gradient)" />
//                         <path
//                             class=move || {
//                                 format!("main-arc stroke-current {} {}", transition(), bar_color)
//                             }
//                             stroke-dashoffset=set_value
//                             stroke-dasharray="452.389"
//                             d="M 0 -72 A 72 72 0 1 1 -4.52 -71.86"
//                             fill="transparent"
//                             stroke-width=bar_width
//                             stroke-linecap="round"
//                         />

//                         // For nice fade out during reset
//                         <path
//                             class=move || { format!("main-arc stroke-current {bar_color}") }
//                             style=reset_bar_animation
//                             stroke-dasharray="452.389"
//                             d="M 0 -72 A 72 72 0 1 1 -4.52 -71.86"
//                             fill="transparent"
//                             stroke-width=bar_width
//                             stroke-linecap="round"
//                         />
//                     </g>
//                 </svg>
//                 <div
//                     class="absolute top-1/2 start-1/2 transform -translate-y-1/2 -translate-x-1/2"
//                     style=reset_icon_animation
//                 >
//                     {children()}
//                 </div>
//             </div>
//         </div>
//     }
// }

#[component]
pub fn CircularProgressBar(
    // Percent value, must be between 0 and 100.
    #[prop(into)] value: Signal<f32>,
    bar_color: &'static str,
    bar_width: u8,
    // Instant reset
    #[prop(into,default = Signal::derive(|| false))] reset: Signal<bool>,
    // Inside the circular bar
    children: Children,
) -> impl IntoView {
    let set_value = move || {
        if reset.get() {
            0.0
        } else {
            value.get().clamp(0.0, 100.0)
        }
    };

    // Trick to reset animation by removing it when ended
    let reset_bar_animation = RwSignal::new("opacity: 0;");
    let reset_icon_animation = RwSignal::new("");
    Effect::new(move |_| {
        if reset.get() {
            reset_bar_animation
                .set("animation: circular-progress-bar-fade-out 0.5s ease-out; animation-fill-mode: both;");
            reset_icon_animation
                .set("animation: circular-progress-bar-glow 0.5s ease; animation-fill-mode: both;");
            set_timeout(
                move || {
                    reset_bar_animation.set("opacity: 0;");
                    reset_icon_animation.set("");
                },
                std::time::Duration::from_millis(500),
            );
        }
    });

    let transition = move || {
        if reset.get() {
            ""
        } else {
            "transition: --progress 0.2s linear;"
        }
    };

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
                        style=move || format!("
                            background: conic-gradient(
                                {bar_color} var(--progress),
                                transparent var(--progress) 100%
                            );
                            {}
                        ",transition())
                        style:--progress=move || format!("{}%", set_value())
                    ></div>

                    // For nice fade out during reset
                    <div
                        class="absolute inset-0 rounded-full"
                        style=move || format!("
                            background: {bar_color};
                            {}
                        ",reset_bar_animation.get())
                    ></div>

                    <div class=format!("absolute inset-{} lg:inset-{bar_width} rounded-full
                            bg-radial from-stone-600 to-zinc-950 to-70%", bar_width/2)></div>
                </div>

                // <svg class="size-full overflow-visible" viewBox="0 0 180 180">
                // <defs>
                // <clipPath id="ring" clip-rule="evenodd">
                // <path d="M0-81A81 81 0 0 1 0 81A81 81 0 0 1 0-81z
                // M0-63A63 63 0 0 1 0 63A63 63 0 0 1 0-63z" />
                // </clipPath>
                // <radialGradient id="inner-gradient" cx="50%" cy="50%" r="50%">
                // <stop offset="0%" stop-color="oklch(44.4% 0.011 73.639)" />
                // <stop offset="100%" stop-color="oklch(14.1% 0.005 285.823)" />
                // </radialGradient>
                // </defs>

                // <g transform="translate(90,90)">
                // <g clip-path="url(#ring)">
                // <circle
                // class="stroke-current text-stone-900"
                // cx="0"
                // cy="2.5"
                // r="72"
                // stroke-width=bar_width
                // />
                // </g>
                // <circle cx="0" cy="0" r="63" fill="url(#inner-gradient)" />
                // <path
                // class=move || {
                // format!("main-arc stroke-current {} {}", transition(), bar_color)
                // }
                // stroke-dashoffset=set_value
                // stroke-dasharray="452.389"
                // d="M 0 -72 A 72 72 0 1 1 -4.52 -71.86"
                // fill="transparent"
                // stroke-width=bar_width
                // stroke-linecap="round"
                // />

                // // For nice fade out during reset
                // <path
                // class=move || { format!("main-arc stroke-current {bar_color}") }
                // style=reset_bar_animation
                // stroke-dasharray="452.389"
                // d="M 0 -72 A 72 72 0 1 1 -4.52 -71.86"
                // fill="transparent"
                // stroke-width=bar_width
                // stroke-linecap="round"
                // />
                // </g>
                // </svg>
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
