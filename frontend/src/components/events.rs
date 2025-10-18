use std::collections::HashMap;

use leptos::{
    ev::{keydown, keyup},
    prelude::*,
};
use leptos_use::{use_document, use_event_listener};

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
        pressed_keys.update(|pressed_keys| {
            pressed_keys.insert(Key::from(ev.key().as_str()), true);
        });
    });

    let _ = use_event_listener(use_document(), keyup, move |ev| {
        pressed_keys
            .write()
            .insert(Key::from(ev.key().as_str()), false);
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
            "Alt" => Key::Alt,
            "Control" => Key::Ctrl,
            "Shift" => Key::Shift,
            "Space" => Key::Space,
            "Enter" => Key::Enter,
            "Escape" => Key::Escape,
            "ArrowUp" => Key::ArrowUp,
            "ArrowDown" => Key::ArrowDown,
            "ArrowLeft" => Key::ArrowLeft,
            "ArrowRight" => Key::ArrowRight,
            s if s.len() == 1 => Key::Character(s.chars().next().unwrap()),
            _ => Key::Unknown,
        }
    }
}
