use leptos::prelude::*;

#[component]
pub fn Number(value: Signal<f64>) -> impl IntoView {
    view! {
        {move || {
            if value.get() >= 1000.0 {
                format!("{:.2e}", value.get())
            } else {
                format!("{}", value.get())
            }
        }}
    }
}
