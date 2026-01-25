use leptos::{prelude::*, web_sys};
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
            value=move || { bind.get().and_then(|value| serde_plain::to_string(&value).ok()) }
            on:input:target=move |ev| bind.set(serde_plain::from_str(&ev.target().value()).ok())

            on:keydown=move |ev| {
                if ev.key() == "Enter" {
                    ev.prevent_default();
                    if let Some(el) = node_ref.get() {
                        let input: web_sys::HtmlInputElement = el;
                        let _ = input.blur();
                    }
                }
            }

            class=move || {
                format!(
                    "w-full px-2 xl:px-4 py-1 xl:py-2 rounded-xl border bg-gray-800 text-white placeholder-gray-400
                    text-sm xl:text-base
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
    #[prop(default = "")] placeholder: &'static str,
    #[prop(default = "any")] step: &'static str,
    bind: RwSignal<Option<T>>,
) -> impl IntoView
where
    T: serde::de::DeserializeOwned
        + serde::ser::Serialize
        + std::cmp::PartialEq
        + Clone
        + Send
        + Sync
        + 'static,
{
    let node_ref = NodeRef::<leptos::html::Input>::new();
    let validation_error = RwSignal::new(None);
    let is_invalid = Memo::new(move |_| validation_error.read().is_some());

    view! {
        <div class="flex flex-col">
            {(!label.is_empty())
                .then(|| {
                    view! {
                        <div class="flex justify-between items-center mb-1">
                            <label for=id class="text-xs xl:text-sm font-medium text-gray-400">
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
                        "w-full px-2 xl:px-4 py-1 xl:py-2 rounded-xl border bg-gray-800 text-white placeholder-gray-400
                        text-sm xl:text-base
                        focus:outline-none focus:ring-2 focus:ring-amber-400 shadow-md {}",
                        if is_invalid.get() {
                            "border-red-500 focus:ring-red-500"
                        } else {
                            "border-gray-700"
                        },
                    )
                }
                placeholder=placeholder
                step=step
                prop:value=move || {
                    bind.get().and_then(|value| serde_plain::to_string(&value).ok())
                }
                on:input:target=move |ev| match serde_plain::from_str(&ev.target().value()) {
                    Ok(v) => {
                        if bind.get().as_ref() != Some(&v) {
                            bind.set(Some(v));
                        }
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
                                                x.split(" Expected valid")
                                                    .next()
                                                    .unwrap_or_default()
                                                    .to_string()
                                            } else {
                                                "Invalid input.".to_string()
                                            }
                                        }
                                        serde_plain::Error::Message(m) => {
                                            m.split(" Expected valid")
                                                .next()
                                                .unwrap_or_default()
                                                .to_string()
                                        }
                                    },
                                ),
                            );
                    }
                }

                on:keydown=move |ev| {
                    if ev.key() == "Enter" {
                        ev.prevent_default();
                        if let Some(el) = node_ref.get() {
                            let input: web_sys::HtmlInputElement = el;
                            let _ = input.blur();
                        }
                    }
                }

                node_ref=node_ref
            />
        </div>
    }
}
