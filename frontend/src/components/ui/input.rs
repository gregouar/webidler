use leptos::prelude::*;
use serde_plain;

#[component]
pub fn Input<T>(
    id: &'static str,
    input_type: &'static str,
    placeholder: &'static str,
    bind: RwSignal<Option<T>>,
    #[prop(optional)] invalid: Option<Signal<bool>>,
    #[prop(optional)] node_ref: NodeRef<leptos::html::Input>,
) -> impl IntoView
where
    T: serde::de::DeserializeOwned + serde::ser::Serialize + Clone + Send + Sync + 'static,
{
    view! {
        <input
            id=id
            type=input_type
            placeholder=placeholder
            value=bind
                .get_untracked()
                .map(|value| serde_plain::to_string(&value).ok())
                .unwrap_or_default()
            on:input:target=move |ev| bind.set(serde_plain::from_str(&ev.target().value()).ok())
            class=move || {
                format!(
                    "w-full px-4 py-2 rounded-xl border bg-gray-800 text-white placeholder-gray-400
                        focus:outline-none focus:ring-2 focus:ring-amber-400 shadow-md {}",
                    if invalid.map(|invalid| invalid.get()).unwrap_or_default() {
                        "border-red-500 focus:ring-red-500"
                    } else {
                        "border-gray-700"
                    },
                )
            }
            node_ref=node_ref
        />
    }
}
#[component]
pub fn ValidatedInput<T>(
    id: &'static str,
    #[prop(default = "")] label: &'static str,
    input_type: &'static str,
    placeholder: &'static str,
    bind: RwSignal<Option<T>>,
) -> impl IntoView
where
    T: serde::de::DeserializeOwned + serde::ser::Serialize + Clone + Send + Sync + 'static,
{
    let validation_error = RwSignal::new(None);
    let is_invalid = Memo::new(move |_| validation_error.read().is_some());

    // err
    //     .to_string()
    //     .split(" Expected valid")
    //     .next()
    //     .unwrap_or_default()
    //     .to_string(),
    view! {
        <div class="flex flex-col">
            {(!label.is_empty())
                .then(|| {
                    view! {
                        <div class="flex justify-between items-center mb-1">
                            <label for=id class="text-sm font-medium text-gray-300">
                                {label}
                            </label>
                            <span class="text-red-500 text-xs">
                                {move || validation_error.get().unwrap_or_default()}
                            </span>
                        </div>
                    }
                })}
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
                value=bind
                    .get_untracked()
                    .map(|value| serde_plain::to_string(&value).ok())
                    .unwrap_or_default()
                on:input:target=move |ev| match serde_plain::from_str(&ev.target().value()) {
                    Ok(v) => {
                        bind.set(Some(v));
                        validation_error.set(None);
                    }
                    Err(err) => {
                        bind.set(None);
                        validation_error
                            .set(
                                Some(
                                    match err {
                                        serde_plain::Error::ImpossibleSerialization(_)
                                        | serde_plain::Error::ImpossibleDeserialization(_) => {
                                            "Invalid input.".to_string()
                                        }
                                        serde_plain::Error::Parse(x, y) => {
                                            if y.starts_with("Expected valid") {
                                                x.to_string()
                                                    .split(" Expected valid")
                                                    .next()
                                                    .unwrap_or_default()
                                                    .to_string()
                                            } else {
                                                "Invalid input.".to_string()
                                            }
                                        }
                                        serde_plain::Error::Message(m) => {
                                            m.to_string()
                                                .split(" Expected valid")
                                                .next()
                                                .unwrap_or_default()
                                                .to_string()
                                        }
                                    },
                                ),
                            );
                    }
                }
            />
        </div>
    }
}
