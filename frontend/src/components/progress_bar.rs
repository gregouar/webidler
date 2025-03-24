use leptos::html::*;
use leptos::prelude::*;

#[component]
pub fn HorizontalProgressBar(
    // Percent value, must be between 0 and 100.
    value: ReadSignal<f32>,
    // Bar color, must be of format "bg-XXXX-NNN"
    bar_color: &'static str,
) -> impl IntoView {
    view! {
        <div class="
            flex w-full
            rounded-full overflow-hidden 
            mt-2 mb-2 
            bg-stone-900 border border-neutral-950 "
        >
            <div
                class={format!(
                    "flex flex-col {} rounded-full transition-all ease-out duration-1000",
                    bar_color
                )}
                style:width=move || format!("{}%", value.get().round())
            ></div>
        </div>
    }

    //     <div class="flex w-full h-2 bg-gray-200 rounded-full overflow-hidden dark:bg-neutral-700" role="progressbar" aria-valuenow="50" aria-valuemin="0" aria-valuemax="100">
    //     <div class="flex flex-col justify-center rounded-full overflow-hidden bg-yellow-500 text-xs text-white text-center whitespace-nowrap transition duration-500" style="width: 50%"></div>
    //   </div>
}

#[component]
pub fn VerticalProgressBar(
    // Percent value, must be between 0 and 100.
    value: ReadSignal<f32>,
    // Bar color, must be of format "bg-XXXX-NNN"
    bar_color: &'static str,
) -> impl IntoView {
    view! {
        <div class="
            flex flex-col flex-nowrap justify-end h-full
            rounded-full overflow-hidden
            ml-2 mr-2
            bg-stone-900 border border-neutral-950
            ">
            <div
                class={format!("{} rounded-full overflow-hidden  transition-all ease-out duration-1000",bar_color)}
                style:height=move || format!("{}%", value.get().round())
            ></div>
        </div>
    }
}

#[component]
pub fn CircularProgressBar(
    // Percent value, must be between 0 and 100.
    value: ReadSignal<f32>,
    // Bar color, must be of format "text-XXXX-NNN"
    bar_color: &'static str,
) -> impl IntoView {
    let set_value = move || 100.0 - value.get().round();
    view! {
        <div class="relative">
            <svg class="size-full -rotate-90" viewBox="0 0 36 36" xmlns="http://www.w3.org/2000/svg">
                <circle cx="18" cy="18" r="16" fill="none" class="stroke-current text-stone-900" stroke-width="2"></circle>
                <circle cx="18" cy="18" r="16" fill="none"
                    class={format!("stroke-current  transition-all ease-out duration-1000 {}",bar_color)}
                    stroke-width="2" stroke-linecap="round"
                    stroke-dashoffset=set_value stroke-dasharray="100"
                ></circle>
            </svg>

            <div class="absolute top-1/2 start-1/2 transform -translate-y-1/2 -translate-x-1/2">
                <span class="text-center text-2xl font-bold text-blue-600 dark:text-blue-500">35%</span>
            </div>
        </div>
    }
}
