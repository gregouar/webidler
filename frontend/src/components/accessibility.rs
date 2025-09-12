use leptos::{prelude::*, web_sys};

#[derive(Clone, Copy)]
pub struct AccessibilityContext {
    on_mobile: bool,
}

impl AccessibilityContext {
    pub fn is_on_mobile(&self) -> bool {
        self.on_mobile
    }
}

pub fn provide_accessibility_context() {
    let window = web_sys::window().unwrap();
    let navigator = window.navigator();
    provide_context(AccessibilityContext {
        on_mobile: navigator.user_agent().unwrap_or_default().contains("Mobi"),
    });
}
