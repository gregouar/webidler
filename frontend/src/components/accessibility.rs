use leptos::{prelude::*, web_sys};

#[derive(Clone, Copy)]
pub struct AccessibilityContext {
    on_mobile: bool,
    is_fullscreen: RwSignal<bool>,
}

impl AccessibilityContext {
    pub fn is_on_mobile(&self) -> bool {
        self.on_mobile
    }

    pub fn is_fullscreen(&self) -> bool {
        self.is_fullscreen.get()
    }

    pub fn go_fullscreen(&self) {
        if !self.is_on_mobile() {
            return;
        }

        let document = web_sys::window().unwrap().document().unwrap();
        if let Some(elem) = document.document_element() {
            let _ = elem.request_fullscreen();
        }
    }

    pub fn exit_fullscreen(&self) {
        if !self.is_on_mobile() {
            return;
        }

        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .exit_fullscreen();
    }

    pub fn toggle_fullscreen(&self) {
        if self.is_fullscreen() {
            self.exit_fullscreen();
        } else {
            self.go_fullscreen();
        }
    }
}

pub fn provide_accessibility_context() {
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let navigator = window.navigator();

    let is_fullscreen = RwSignal::new(document.fullscreen_element().is_some());

    let closure = web_sys::wasm_bindgen::prelude::Closure::<dyn FnMut(_)>::wrap(Box::new({
        move |_ev: web_sys::Event| {
            is_fullscreen.set(document.fullscreen_element().is_some());
        }
    }));

    let document = window.document().unwrap();
    document
        .add_event_listener_with_callback(
            "fullscreenchange",
            web_sys::wasm_bindgen::JsCast::unchecked_ref(closure.as_ref()),
        )
        .unwrap();
    closure.forget();

    provide_context(AccessibilityContext {
        on_mobile: navigator.user_agent().unwrap_or_default().contains("Mobi"),
        is_fullscreen,
    });
}
