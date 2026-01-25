use leptos::{prelude::*, wasm_bindgen::JsCast};
use regex::Regex;
use serde::{de::DeserializeOwned, Serialize};

#[component]
pub fn JsonEditor<T>(label: &'static str, value: RwSignal<T>) -> impl IntoView
where
    T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
{
    let textarea_ref = NodeRef::<leptos::html::Textarea>::new();
    let pre_ref = NodeRef::<leptos::html::Pre>::new();
    let gutter_ref = NodeRef::<leptos::html::Div>::new();

    let text = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);

    // Sync FROM model → editor
    Effect::new({
        let textarea_ref = textarea_ref.clone();
        let gutter_ref = gutter_ref.clone();
        move || {
            let json = serde_json::to_string_pretty(&value.get()).unwrap();
            text.set(json.clone());
            error.set(None);

            if let Some(el) = textarea_ref.get() {
                el.set_value(&json);
            }

            if let Some(gutter) = gutter_ref.get() {
                let lines = json.lines().count();
                let gutter_text = (1..=lines)
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join("\n");
                gutter.set_inner_text(&gutter_text);
            }
        }
    });

    // Input handler
    let on_input = move |ev: leptos::ev::Event| {
        let input = event_target_value(&ev);
        text.set(input.clone());

        match serde_json::from_str::<T>(&input) {
            Ok(parsed) => {
                value.set(parsed);
                error.set(None);
            }
            Err(e) => error.set(Some(e.to_string())),
        }

        if let Some(gutter) = gutter_ref.get() {
            let lines = input.lines().count();
            let gutter_text = (1..=lines)
                .map(|n| n.to_string())
                .collect::<Vec<_>>()
                .join("\n");
            gutter.set_inner_text(&gutter_text);
        }
    };

    // Scroll sync
    let on_scroll = move |ev: leptos::ev::Event| {
        if let Some(ta) = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok())
        {
            if let Some(pre) = pre_ref.get() {
                pre.set_scroll_top(ta.scroll_top());
                pre.set_scroll_left(ta.scroll_left());
            }
            if let Some(gutter) = gutter_ref.get() {
                gutter.set_scroll_top(ta.scroll_top());
            }
        }
    };

    view! {
        <style>
            ".json-key { color: #fbbf24; }
             .json-string { color: #34d399; }
             .json-number { color: #60a5fa; }
             .json-bool { color: #f472b6; }
             .json-null { color: #a78bfa; }
             .json-editor-gutter { color: #888; text-align: right; user-select: none; padding-right: 4px; }"
        </style>

        <div class="flex flex-col space-y-1 text-left">
            <label class="text-xs font-medium text-gray-400">{label}</label>

            <div class="relative flex w-full h-64">
                <div
                    node_ref=gutter_ref
                    class="
                    json-editor-gutter
                    flex-shrink-0
                    font-mono text-sm leading-5
                    p-2 h-64 overflow-y-hidden
                    bg-gray-800 rounded-l-lg
                    text-right
                    "
                />

                <div class="relative flex-1 h-full">
                    <pre
                        node_ref=pre_ref
                        class="
                        absolute inset-0 overflow-auto
                        pointer-events-none
                        font-mono text-sm leading-5
                        p-2 whitespace-pre-wrap break-words
                        bg-gray-900 rounded-r-lg
                        "
                        inner_html=move || highlight_json(&text.get())
                    />
                    <textarea
                        node_ref=textarea_ref
                        class=move || {
                            format!(
                                "relative w-full h-full font-mono text-sm leading-5 p-2
                    bg-transparent resize-none
                    text-transparent caret-amber-400
                    border rounded-r-lg focus:outline-none
                    {}",
                                if error.get().is_some() {
                                    "border-red-500"
                                } else {
                                    "border-gray-700"
                                },
                            )
                        }
                        spellcheck="false"
                        on:input=on_input
                        on:scroll=on_scroll
                    />
                </div>
            </div>

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

fn highlight_json(json: &str) -> String {
    let text = html_escape::encode_text(json);

    // Regex: capture any quoted string, optionally followed by a colon
    let re = Regex::new(r#""([^"\\]*(?:\\.[^"\\]*)*)"(?:\s*(:))?(\s*)"#).unwrap();

    let highlighted = re.replace_all(&text, |caps: &regex::Captures| {
        let trailing_ws = &caps[3]; // preserve spaces/newlines

        if caps.get(2).is_some() {
            // Key
            format!(
                r#"<span class="json-key">"{}"</span>:{}"#,
                &caps[1], trailing_ws
            )
        } else {
            // String value
            format!(
                r#"<span class="json-string">"{}"</span>{}"#,
                &caps[1], trailing_ws
            )
        }
    });

    // Numbers
    let re_number = Regex::new(r"\b-?\d+(\.\d+)?([eE][+-]?\d+)?\b").unwrap();
    let highlighted = re_number.replace_all(&highlighted, r#"<span class="json-number">$0</span>"#);

    // Booleans and null
    let re_bool = Regex::new(r"\b(true|false|null)\b").unwrap();
    let highlighted = re_bool.replace_all(&highlighted, r#"<span class="json-bool">$0</span>"#);

    highlighted.to_string()
}
