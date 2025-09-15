use leptos::{prelude::*, web_sys};

#[derive(Clone, Copy)]
pub enum ScreenOrientation {
    Horizontal,
    Vertical,
}

#[derive(Clone, Copy)]
pub struct AccessibilityContext {
    on_mobile: bool,
}

impl AccessibilityContext {
    pub fn is_on_mobile(&self) -> bool {
        self.on_mobile
    }

    pub fn is_fullscreen(&self) -> bool {
        web_sys::window()
            .unwrap()
            .document()
            .unwrap()
            .fullscreen_element()
            .is_some()
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
}

pub fn provide_accessibility_context() {
    let window = web_sys::window().unwrap();
    let navigator = window.navigator();
    provide_context(AccessibilityContext {
        on_mobile: navigator.user_agent().unwrap_or_default().contains("Mobi"),
    });
}
