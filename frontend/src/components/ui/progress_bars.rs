use leptos::html::*;
use leptos::prelude::*;

#[component]
pub fn HorizontalProgressBar(
    /// Percent value, must be between 0 and 100.
    #[prop(into)]
    value: Signal<f32>,
    /// Bar color, must be of format "bg-XXXX-NNN"
    bar_color: &'static str,
    /// Text
    #[prop(optional)]
    text: Option<String>, // TODO: Dynamic?
) -> impl IntoView {
    view! {
        <div class="
                relative flex w-full
                rounded-lg overflow-hidden 
                bg-stone-900 border border-neutral-950 
                shadow-md
            ">
            <div class="absolute inset-0 flex items-center justify-center text-white text-sm pointer-events-none">
                {text}
            </div>

            <div
                class={format!(
                    "flex flex-col {} rounded-lg transition-all ease duration-500",
                    bar_color
                )}
                style:width=move || format!("{}%", value.get().max(0.0).min(100.0).round())
            ></div>
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
                class={format!("{} rounded-lg overflow-hidden transition-all ease duration-500",bar_color)}
                style:height=move || format!("{}%", value.get().max(0.0).min(100.0).round())
                style:-webkit-mask="linear-gradient(#fff 0 0)"
            ></div>
        </div>
    }
}

#[component]
pub fn CircularProgressBar(
    // Percent value, must be between 0 and 100.
    #[prop(into)] value: Signal<f32>,
    // Bar color, must be of format "text-XXXX-NNN"
    bar_color: &'static str,
    // Width of the progress bar
    #[prop(default = 2)] bar_width: u16,
    // Instant reset
    #[prop(default = Signal::derive(|| false))] reset: Signal<bool>,
    // Inside the circular bar
    children: Children,
) -> impl IntoView {
    let bar_width = bar_width * 5;
    let set_value = move || 452.389 - value.get().max(0.0).min(100.0) * 452.389 / 100.0;
    let transition = move || {
        if reset.get() {
            "transition-none"
        } else {
            "transition-all ease-linear duration-200 "
        }
    };
    view! {
        <div>
            <div class="relative drop-shadow-lg">
            <svg  class="size-full" viewBox="0 0 180 180">
                <defs>
                    <filter id="blur" filterUnits="userSpaceOnUse" x="-90" y="-90"
                            width="180" height="180">
                        <feGaussianBlur in="SourceGraphic" stdDeviation="1" />
                    </filter>
                    <clipPath id="ring" clip-rule="evenodd">
                        <path d="M0-81A81 81 0 0 1 0 81A81 81 0 0 1 0-81z
                                M0-63A63 63 0 0 1 0 63A63 63 0 0 1 0-63z" />
                    </clipPath>
                </defs>

                <g transform="translate(90,90)">
                    <g clip-path="url(#ring)">
                        <rect x="-85" y="-85" width="170" height="170" fill="stone-950"
                                stroke="none" />
                        <circle class="stroke-current text-stone-900" cx="0" cy="2.5" r="72" stroke-width=bar_width
                                fill="none" filter="url(#blur)"/>
                    </g>
                    <path class=move || {format!("main-arc stroke-current {} {}", transition(),bar_color)}
                        stroke-dashoffset=set_value stroke-dasharray="452.389"
                        d="M 0 -72 A 72 72 0 1 1 -4.52 -71.86"
                        fill="transparent" stroke-width=bar_width stroke=bar_color
                        stroke-linecap="round" />
                </g>
            </svg>
                <div class="absolute top-1/2 start-1/2 transform -translate-y-1/2 -translate-x-1/2">
                    {children()}
                </div>
            </div>
        </div>
    }
}

// #[component]
// pub fn CircularProgressBar(
//     // Percent value, must be between 0 and 100.
//     value: ReadSignal<f32>,
//     // Bar color, must be of format "text-XXXX-NNN"
//     bar_color: &'static str,
//     // Width of the progress bar
//     #[prop(default = 2)] bar_width: u16,
//     // Inside the circular bar
//     children: Children,
// ) -> impl IntoView {
//     let set_value = move || 100.0 - value.get().round();
//     view! {
//         <div>
//             <div class="relative drop-shadow-lg">
//                 <svg class="size-full -rotate-90" viewBox="0 0 36 36">
//                     <circle cx="18" cy="18" r="16" fill="none" class="stroke-current text-stone-900" stroke-width=bar_width></circle>
//                     <circle cx="18" cy="18" r="16" fill="none"
//                         class={format!("stroke-current  transition-all ease-out duration-1000 {}",bar_color)}
//                         stroke-width=bar_width stroke-linecap="round"
//                         stroke-dashoffset=set_value stroke-dasharray="100"
//                     ></circle>
//                 </svg>

//                 <div class="absolute top-1/2 start-1/2 transform -translate-y-1/2 -translate-x-1/2">
//                     {children()}
//                 </div>
//             </div>
//         </div>
//     }
// }
