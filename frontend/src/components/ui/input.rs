use leptos::prelude::*;
use serde_json;

#[component]
pub fn Input(
    id: &'static str,
    input_type: &'static str,
    placeholder: &'static str,
) -> impl IntoView {
    view! {
        <input
            id=id
            type=input_type
            placeholder=placeholder
            class="w-full px-4 py-2 rounded-xl border border-gray-700 bg-gray-800 text-white placeholder-gray-400
            focus:outline-none focus:ring-2 focus:ring-amber-400 shadow-md"
        />
    }
}

#[component]
pub fn ValidatedInput<T>(
    id: &'static str,
    input_type: &'static str,
    placeholder: &'static str,
    bind: RwSignal<Option<T>>,
) -> impl IntoView
where
    T: serde::de::DeserializeOwned + Clone + Send + Sync + 'static,
{
    let validation_error = RwSignal::new(None);
    let is_invalid = Memo::new(move |_| validation_error.read().is_some());

    view! {
        <div class="flex flex-col">
            <input
                id=id
                type=input_type
                class=move || {
                    format!(
                        "w-full px-4 py-2 rounded-xl border bg-gray-800 text-white placeholder-gray-400
                    focus:outline-none focus:ring-2 focus:ring-amber-400 shadow-md {}",
                        if is_invalid.get() {
                            "border-red-500 focus:ring-red-500"
                        } else {
                            "border-gray-700"
                        },
                    )
                }
                placeholder=placeholder
                on:input:target=move |ev| match serde_json::from_str(&ev.target().value()) {
                    Ok(v) => {
                        bind.set(Some(v));
                        validation_error.set(None);
                    }
                    Err(err) => {
                        bind.set(None);
                        validation_error.set(Some(err.to_string()));
                    }
                }
            />
            <div class="min-h-[1.25rem] text-red-500 text-sm mt-1">
                {move || {
                    view! { {validation_error} }
                }}
            </div>
        </div>
    }
}
