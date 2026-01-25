use leptos::wasm_bindgen::{closure::Closure, JsCast};
use leptos::{prelude::*, web_sys};
use serde::de::DeserializeOwned;
// use web_sys::{
//     wasm_bindgen::{prelude::Closure, JsCast},
//     FileReader, HtmlInputElement,
// };

/// Generic file loader: loads JSON into a signal
pub fn use_json_loader<T: 'static + DeserializeOwned + Sync + Send>(
) -> (RwSignal<Option<T>>, impl Fn(web_sys::Event)) {
    let data = RwSignal::new(None::<T>);

    let on_file_change = {
        let data = data.clone();
        move |event: web_sys::Event| {
            let input: web_sys::HtmlInputElement = event.target().unwrap().dyn_into().unwrap();

            if let Some(file) = input.files().and_then(|f| f.get(0)) {
                let reader = web_sys::FileReader::new().unwrap();
                let data = data.clone();

                // Closure for when reading is done
                let onload = Closure::once_into_js({
                    let reader = reader.clone();
                    move |_e: web_sys::ProgressEvent| {
                        let text = reader.result().unwrap().as_string().unwrap();
                        let parsed: T = serde_json::from_str(&text).unwrap();
                        data.set(Some(parsed));
                    }
                });

                reader.set_onload(Some(onload.unchecked_ref()));
                reader.read_as_text(&file).unwrap();
            }
        }
    };

    (data, on_file_change)
}

// use leptos::*;
// use web_sys::FileReader;

// #[component]
// fn FileLoader(cx: Scope) -> impl IntoView {
//     let data = create_signal(cx, None::<GameData>);

//     let on_file_change = move |ev: web_sys::Event| {
//         let input: web_sys::HtmlInputElement = ev.target().unwrap().dyn_into().unwrap();
//         if let Some(file) = input.files().and_then(|files| files.get(0)) {
//             let reader = FileReader::new().unwrap();
//             let data = data.clone();
//             let onload = Closure::once_into_js(move |e: web_sys::ProgressEvent| {
//                 let text = reader.result().unwrap().as_string().unwrap();
//                 let game: GameData = serde_json::from_str(&text).unwrap();
//                 data.set(Some(game));
//             });
//             reader.set_onload(Some(onload.unchecked_ref()));
//             reader.read_as_text(&file).unwrap();
//         }
//     };

//     view! {
//         <input type="file" on:change=on_file_change />
//         {move || {
//             if let Some(d) = &*data.get() {
//                 view! { <p>{format!("Loaded game: {} (level {})", d.name, d.level)}</p> }.into_any()
//             } else {
//                 view! { <p>"No file loaded."</p> }.into_view(into_any)
//             }
//         }}
//     }
// }
