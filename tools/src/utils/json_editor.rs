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

    let text = RwSignal::new(String::new());
    let error = RwSignal::new(None::<String>);

    // Sync FROM model → editor (imperative!)
    Effect::new({
        let textarea_ref = textarea_ref.clone();
        move || {
            let json = serde_json::to_string_pretty(&value.get()).unwrap();
            text.set(json.clone());
            error.set(None);

            if let Some(el) = textarea_ref.get() {
                el.set_value(&json);
            }
        }
    });

    // Input handler (editor → model)
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

    // Scroll sync
    let on_scroll = move |ev: leptos::ev::Event| {
        // let ta = ev.target_unchecked::<web_sys::HtmlElement>();
        if let Some(ta) = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlElement>().ok())
        {
            if let Some(pre) = pre_ref.get() {
                pre.set_scroll_top(ta.scroll_top());
                pre.set_scroll_left(ta.scroll_left());
            }
        }
    };

    view! {
        <style>
            ".json-key {
            color: #fbbf24; /* amber */
            }
            
            .json-string {
            color: #34d399; /* green */
            }
            
            .json-number {
            color: #60a5fa; /* blue */
            }
            
            .json-bool {
            color: #f472b6; /* pink */
            }
            
            .json-null {
            color: #a78bfa; /* violet */
            }
            
            textarea::selection {
            background: rgba(251, 191, 36, 0.3);
            }"
        </style>
        <div class="flex flex-col space-y-1">
            <label class="text-xs font-medium text-gray-400">{label}</label>

            <div class="relative text-left">
                <pre
                    node_ref=pre_ref
                    class="
                    absolute inset-0 overflow-auto
                    pointer-events-none
                    font-mono text-sm leading-5
                    p-2 whitespace-pre-wrap break-words
                    bg-gray-900 rounded-lg
                    "
                    inner_html=move || highlight_json(&text.get())
                />

                <textarea
                    node_ref=textarea_ref
                    class=move || {
                        format!(
                            "relative w-full h-64 font-mono text-sm leading-5 p-2
                             bg-transparent resize-none
                             text-transparent caret-amber-400
                             border rounded-lg focus:outline-none
                             {}",
                            if error.get().is_some() { "border-red-500" } else { "border-gray-700" },
                        )
                    }
                    spellcheck="false"
                    on:input=on_input
                    on:scroll=on_scroll
                />
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

// fn highlight_json(json: &str) -> String {
//     use serde_json::Value;

//     let Ok(value) = serde_json::from_str::<Value>(json) else {
//         // fallback: escape HTML only
//         return html_escape::encode_text(json).to_string();
//     };

//     fn render(v: &Value, out: &mut String, indent: usize) {
//         let pad = "  ".repeat(indent);

//         match v {
//             Value::Object(map) => {
//                 out.push_str("{\n");
//                 let len = map.len();

//                 for (i, (k, v)) in map.iter().enumerate() {
//                     out.push_str(&pad);
//                     out.push_str("  ");

//                     // key
//                     out.push_str(r#"<span class="json-key">""#);
//                     out.push_str(&html_escape::encode_text(k));
//                     out.push_str(r#""</span>: "#);

//                     render(v, out, indent + 1);

//                     if i + 1 < len {
//                         out.push(',');
//                     }
//                     out.push('\n');
//                 }

//                 out.push_str(&pad);
//                 out.push('}');
//             }

//             Value::Array(arr) => {
//                 out.push_str("[\n");
//                 let len = arr.len();

//                 for (i, v) in arr.iter().enumerate() {
//                     out.push_str(&pad);
//                     out.push_str("  ");
//                     render(v, out, indent + 1);

//                     if i + 1 < len {
//                         out.push(',');
//                     }
//                     out.push('\n');
//                 }

//                 out.push_str(&pad);
//                 out.push(']');
//             }

//             Value::String(s) => {
//                 out.push_str(r#"<span class="json-string">""#);
//                 out.push_str(&html_escape::encode_text(s));
//                 out.push_str(r#""</span>"#);
//             }

//             Value::Number(n) => {
//                 out.push_str(r#"<span class="json-number">"#);
//                 out.push_str(&n.to_string());
//                 out.push_str("</span>");
//             }

//             Value::Bool(b) => {
//                 out.push_str(r#"<span class="json-bool">"#);
//                 out.push_str(if *b { "true" } else { "false" });
//                 out.push_str("</span>");
//             }

//             Value::Null => {
//                 out.push_str(r#"<span class="json-null">null</span>"#);
//             }
//         }
//     }

//     let mut out = String::new();
//     render(&value, &mut out, 0);
//     out
// }

// #[component]
// pub fn JsonEditor<T>(label: &'static str, value: RwSignal<T>) -> impl IntoView
// where
//     T: Serialize + DeserializeOwned + Clone + Send + Sync + 'static,
// {
//     let text = RwSignal::new(String::new());
//     let error = RwSignal::new(None::<String>);

//     Effect::new({
//         let value = value.clone();
//         move || {
//             let json = serde_json::to_string_pretty(&value.get()).unwrap();
//             text.set(json);
//             error.set(None);
//         }
//     });

//     let on_input = move |ev: leptos::ev::Event| {
//         let input = event_target_value(&ev);
//         text.set(input.clone());

//         match serde_json::from_str::<T>(&input) {
//             Ok(parsed) => {
//                 value.set(parsed);
//                 error.set(None);
//             }
//             Err(e) => {
//                 error.set(Some(e.to_string()));
//             }
//         }
//     };

//     view! {
//         <div class="flex flex-col space-y-1">
//             <label class="text-xs font-medium text-gray-400">{label}</label>

//             <div class="relative text-left">
//                 <pre
//                     class="absolute inset-0 overflow-auto pointer-events-none
//                     font-mono text-sm p-2 whitespace-pre-wrap
//                     bg-gray-900 rounded-lg"
//                     inner_html=move || highlight_json(&text.get())
//                 />

//                 <textarea
//                     class=move || {
//                         format!(
//                             "relative w-full h-64 font-mono text-sm p-2
//                bg-transparent text-transparent caret-white
//                border rounded-lg {}",
//                             if error.get().is_some() { "border-red-500" } else { "border-gray-700" },
//                         )
//                     }
//                     spellcheck="false"
//                     prop:value=move || text.get()
//                     on:input=on_input
//                 />
//             </div>

//             {move || {
//                 error
//                     .get()
//                     .map(|e| {
//                         view! { <div class="text-xs text-red-400 whitespace-pre-wrap">{e}</div> }
//                     })
//             }}
//         </div>
//     }
// }

// fn highlight_json(json: &str) -> String {
//     let mut out = String::new();
//     let mut in_string = false;

//     for c in json.chars() {
//         match c {
//             '"' => {
//                 in_string = !in_string;
//                 out.push_str(r#"<span class="text-green-400">"</span>"#);
//             }
//             ':' if !in_string => out.push_str(r#"<span class="text-gray-400">:</span>"#),
//             '{' | '}' | '[' | ']' if !in_string => {
//                 out.push_str(&format!(r#"<span class="text-amber-400">{}</span>"#, c))
//             }
//             _ => out.push(c),
//         }
//     }

//     out
// }
