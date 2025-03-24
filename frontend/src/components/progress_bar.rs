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
                    "flex flex-col {} rounded-full transition duration-500",
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
                class={format!("{} rounded-full overflow-hidden",bar_color)}
                style:height=move || format!("{}%", value.get().round())
            ></div>
        </div>
    }
}
