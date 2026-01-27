use leptos::wasm_bindgen::{closure::Closure, JsCast, JsValue};
use leptos::{prelude::*, web_sys};
use serde::de::DeserializeOwned;
use serde::Serialize;

pub fn use_json_loader<T: 'static + DeserializeOwned + Sync + Send>(
) -> (RwSignal<Option<T>>, impl Fn(web_sys::Event)) {
    let data = RwSignal::new(None::<T>);

    let on_file_change = {
        move |event: web_sys::Event| {
            let input: web_sys::HtmlInputElement = event.target().unwrap().dyn_into().unwrap();

            if let Some(file) = input.files().and_then(|f| f.get(0)) {
                let reader = web_sys::FileReader::new().unwrap();

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

pub fn save_json<T: Serialize>(data: &T, filename: &str) {
    let json = serde_json::to_string_pretty(data).unwrap();

    let array = web_sys::js_sys::Array::new();
    array.push(&JsValue::from_str(&json));
    let blob_property = web_sys::BlobPropertyBag::new();
    blob_property.set_type("application/json");
    let blob = web_sys::Blob::new_with_str_sequence_and_options(&array, &blob_property).unwrap();

    let url = web_sys::Url::create_object_url_with_blob(&blob).unwrap();
    let document = web_sys::window().unwrap().document().unwrap();
    let a: web_sys::HtmlAnchorElement = document.create_element("a").unwrap().dyn_into().unwrap();

    a.set_href(&url);
    a.set_download(filename);

    document.body().unwrap().append_child(&a).unwrap();
    a.click();
    document.body().unwrap().remove_child(&a).unwrap();

    web_sys::Url::revoke_object_url(&url).unwrap();
}

// pub fn save_json_with_fsapi<T: Serialize + 'static>(data: &T) {
//     let json = serde_json::to_string_pretty(data).unwrap();

//     spawn_local(async move {
//         let opts = web_sys::FileSystemCreateWritableOptions::new();
//         if let Ok(handle) = window().unwrap().show_save_file_picker().await {
//             let writable = handle.create_writable_with_options(&opts).await.unwrap();
//             writable.write_with_str(&json).await.unwrap();
//             writable.close().await.unwrap();
//         }
//     });
// }
