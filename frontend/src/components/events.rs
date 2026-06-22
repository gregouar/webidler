use std::collections::HashMap;

use leptos::{
    ev::{keydown, keyup, visibilitychange},
    prelude::*,
    web_sys::{
        Element, HtmlInputElement, HtmlTextAreaElement, wasm_bindgen::JsCast, wasm_bindgen::JsValue,
    },
};
use leptos_use::{use_document, use_event_listener};

pub fn keyboard_event_key(ev: &web_sys::KeyboardEvent) -> Option<String> {
    keyboard_event_string_property(ev, "key")
}

fn keyboard_event_code(ev: &web_sys::KeyboardEvent) -> Option<String> {
    keyboard_event_string_property(ev, "code")
}

fn keyboard_event_string_property(ev: &web_sys::KeyboardEvent, property: &str) -> Option<String> {
    js_sys::Reflect::get(ev.as_ref(), &JsValue::from_str(property))
        .ok()
        .and_then(|value| value.as_string())
}

#[derive(Clone, Copy)]
pub struct EventsContext {
    pub pressed_keys: RwSignal<HashMap<Key, bool>>,
}

impl EventsContext {
    pub fn key_pressed(&self, key: Key) -> bool {
        self.pressed_keys
            .with(|set| set.get(&key).cloned().unwrap_or_default())
    }
}

pub fn provide_events_context() {
    let pressed_keys = RwSignal::new(HashMap::new());

    let _ = use_event_listener(use_document(), keydown, move |ev| {
        if is_text_input_target(&ev) {
            return;
        }

        if let Ok(Some(selection)) = window().get_selection()
            && !selection.is_collapsed()
        {
            return;
        }

        let code = keyboard_event_code(&ev);
        if ev.alt_key()
            || code.as_deref() == Some("AltLeft")
            || code.as_deref() == Some("AltRight")
            || ev.ctrl_key()
        {
            ev.prevent_default();
            ev.stop_propagation();
        }

        pressed_keys.update(|pressed_keys| {
            pressed_keys.insert(
                keyboard_event_key(&ev)
                    .as_deref()
                    .map(Key::from)
                    .unwrap_or(Key::Unknown),
                true,
            );
        });
    });

    let _ = use_event_listener(use_document(), keyup, move |ev| {
        // if is_text_input_target(&ev) {
        //     return;
        // }

        let code = keyboard_event_code(&ev);
        if !is_text_input_target(&ev)
            && (ev.alt_key()
                || code.as_deref() == Some("AltLeft")
                || code.as_deref() == Some("AltRight")
                || ev.ctrl_key())
        {
            ev.prevent_default();
            ev.stop_propagation();
        }

        pressed_keys.write().insert(
            keyboard_event_key(&ev)
                .as_deref()
                .map(Key::from)
                .unwrap_or(Key::Unknown),
            false,
        );
    });

    let _ = use_event_listener(use_document(), visibilitychange, move |_| {
        pressed_keys.update(|keys| keys.clear());
    });

    provide_context(EventsContext { pressed_keys });
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Key {
    Alt,
    Ctrl,
    Shift,
    Space,
    Enter,
    Escape,
    Delete,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Character(char),
    Unknown,
}

impl From<&str> for Key {
    fn from(code: &str) -> Self {
        match code {
            "Alt" | "AltLeft" | "AltRight" => Key::Alt,
            "Control" | "ControlLeft" | "ControlRight" => Key::Ctrl,
            "Shift" => Key::Shift,
            "Space" => Key::Space,
            "Enter" => Key::Enter,
            "Escape" => Key::Escape,
            "ArrowUp" => Key::ArrowUp,
            "ArrowDown" => Key::ArrowDown,
            "ArrowLeft" => Key::ArrowLeft,
            "ArrowRight" => Key::ArrowRight,
            "Delete" => Key::Delete,
            s if s.len() == 1 => Key::Character(s.chars().next().unwrap()),
            _ => Key::Unknown,
        }
    }
}

fn is_text_input_target(ev: &web_sys::KeyboardEvent) -> bool {
    ev.target()
        .and_then(|t| t.dyn_into::<Element>().ok())
        .map(|el| {
            el.dyn_ref::<HtmlInputElement>().is_some()
                || el.dyn_ref::<HtmlTextAreaElement>().is_some()
                || el.get_attribute("contenteditable").as_deref() == Some("true")
        })
        .unwrap_or(false)
}
