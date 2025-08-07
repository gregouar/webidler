use leptos::prelude::*;
use leptos::wasm_bindgen::{closure::Closure, JsCast, JsValue};
use leptos::web_sys::{js_sys, window};
use leptos::*;

#[component]
pub fn Captcha(token: RwSignal<Option<String>>) -> impl IntoView {
    let turnstile_ref = NodeRef::new();

    Effect::new({
        let turnstile_ref = turnstile_ref.clone();
        move || {
            // Wrap the rust closure as JS closure
            let closure = Closure::wrap(Box::new(move |token_js: JsValue| {
                if let Some(token_str) = token_js.as_string() {
                    token.set(Some(token_str));
                }
            }) as Box<dyn FnMut(JsValue)>);

            // Register the closure as a JS callback so that we can refer to it in the turnstile widget
            let _ = js_sys::Reflect::set(
                &window().unwrap(),
                &JsValue::from_str("on_turnstile_success"),
                closure.as_ref().unchecked_ref(),
            );

            // Trigger turnstile on mount (effects are run on mount)
            if let Some(elem) = turnstile_ref.get() {
                let window = web_sys::window().unwrap();
                if let Ok(turnstile) =
                    js_sys::Reflect::get(&window, &JsValue::from_str("turnstile"))
                {
                    if let Ok(render_fn) =
                        js_sys::Reflect::get(&turnstile, &JsValue::from_str("render"))
                    {
                        if let Some(render_fn) = render_fn.dyn_ref::<js_sys::Function>() {
                            let _ = render_fn.call1(&JsValue::NULL, &JsValue::from(elem));
                        }
                    }
                }
            }

            closure.forget();
        }
    });

    view! {
        <div
            node_ref=turnstile_ref
            class="cf-turnstile"
            data-sitekey="0x4AAAAAABoSog3mlP1Ok1U9"
            data-theme="dark"
            data-callback="on_turnstile_success"
        />
    }
    // TODO: Replace sitekey by env setting
}
