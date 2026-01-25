use leptos::prelude::*;
use serde::{de::DeserializeOwned, Serialize};

#[component]
pub fn JsonEditor<T>(label: &'static str, value: RwSignal<T>) -> impl IntoView
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    let text = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);

    Effect::new({
        let value = value.clone();
        move || {
            let json = serde_json::to_string_pretty(&value.get()).unwrap();
            text.set(json);
            error.set(None);
        }
    });

    let on_input = move |ev: leptos::ev::Event| {
        let input = event_target_value(&ev);
        text.set(input.clone());

        match serde_json::from_str::<T>(&input) {
            Ok(parsed) => {
                value.set(parsed);
                error.set(None);
            }
            Err(e) => {
                error.set(Some(e.to_string()));
            }
        }
    };

    view! {
        <div class="flex flex-col space-y-1">
            <label class="text-xs font-medium text-gray-400">{label}</label>

            <textarea
                class=move || {
                    format!(
                        "w-full h-64 font-mono text-sm rounded-lg p-2 bg-gray-900 text-gray-100 border {}",
                        if error.get().is_some() { "border-red-500" } else { "border-gray-700" },
                    )
                }
                spellcheck="false"
                prop:value=move || text.get()
                on:input=on_input
            />

            {move || {
                error
                    .get()
                    .map(|e| {
                        view! { <div class="text-xs text-red-400 whitespace-pre-wrap">{e}</div> }
                    })
            }}
        </div>
    }
}
