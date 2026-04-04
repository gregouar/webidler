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
    let prop_value = RwSignal::new(String::new());
    Effect::new(move || {
        if let Some(value) = bind.get() {
            prop_value.set(serde_plain::to_string(&value).ok().unwrap_or_default())
        }
    });

    view! {
        <input
            id=id
            type=input_type
            placeholder=placeholder
            prop:value=move || prop_value.get()
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
                    "w-full px-2 xl:px-4 py-1 xl:py-2 rounded-[4px] xl:rounded-[6px]
                    border text-white placeholder:text-zinc-500
                    text-sm xl:text-base
                    shadow-[0_4px_10px_rgba(0,0,0,0.35),0_1px_0_rgba(26,17,10,0.9),inset_0_2px_6px_rgba(0,0,0,0.42),inset_0_1px_0_rgba(255,255,255,0.03)]
                    focus:outline-none {}",
                    if invalid.map(|invalid| invalid.get()).unwrap_or_default() {
                        "border-[#9b453c] focus:border-[#c35d52]"
                    } else {
                        "border-[#6c5329] focus:border-[#a27f46]"
                    },
                )
            }
            style="
            background-image:
                linear-gradient(180deg, rgba(255,255,255,0.03), rgba(0,0,0,0.08)),
                linear-gradient(180deg, rgba(14,14,17,0.94), rgba(31,29,35,0.98));
            background-size: auto, auto;
            background-position: center, center;
            background-blend-mode: screen, normal;
            "
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

    let prop_value = RwSignal::new(String::new());
    Effect::new(move || {
        if let Some(value) = bind.get() {
            validation_error.set(None);
            prop_value.set(serde_plain::to_string(&value).ok().unwrap_or_default())
        } else if validation_error.read().is_none() {
            prop_value.set("".into());
        }
    });

    view! {
        <div class="flex flex-col">
            {(!label.is_empty())
                .then(|| {
                    view! {
                        <div class="flex justify-between items-center mb-1">
                            <label for=id class="text-xs xl:text-sm font-medium text-zinc-400">
                                {label}
                            </label>
                            <span class="text-[#c66d61] text-xs">
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
                        "w-full px-2 xl:px-4 py-1 xl:py-2 rounded-[4px] xl:rounded-[6px] border
                        text-white placeholder:text-zinc-500
                        text-sm xl:text-base
                        shadow-[0_4px_10px_rgba(0,0,0,0.35),0_1px_0_rgba(26,17,10,0.9),inset_0_2px_6px_rgba(0,0,0,0.42),inset_0_1px_0_rgba(255,255,255,0.03)]
                        focus:outline-none {}",
                        if is_invalid.get() {
                            "border-[#9b453c] focus:border-[#c35d52]"
                        } else {
                            "border-[#6c5329] focus:border-[#a27f46]"
                        },
                    )
                }
                style="
                background-image:
                    linear-gradient(180deg, rgba(255,255,255,0.03), rgba(0,0,0,0.08)),
                    linear-gradient(180deg, rgba(14,14,17,0.94), rgba(31,29,35,0.98));
                background-size: auto, auto;
                background-position: center, center;
                background-blend-mode: screen, normal;
                "
                placeholder=placeholder
                step=step
                prop:value=move || prop_value.get()
                on:input:target=move |ev| match serde_plain::from_str(&ev.target().value()) {
                    Ok(v) => {
                        if bind.get_untracked().as_ref() != Some(&v) {
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
