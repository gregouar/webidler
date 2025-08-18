use chrono::{DateTime, Utc};
use std::time::Duration;

use leptos::{prelude::*, wasm_bindgen::JsValue, web_sys::js_sys::Date};

#[component]
pub fn Number(value: Signal<f64>) -> impl IntoView {
    view! { {move || { format_number(value.get()) }} }
}

pub fn format_number(value: f64) -> String {
    let value = value.round();

    if value < 0.0 {
        return format!("-{}", format_number(-value));
    }

    if value >= 1_000_000.0 {
        format!("{value:.2e}")
    } else {
        comma_format(value)
    }
}

fn comma_format(value: f64) -> String {
    let value_str = value.round().to_string();
    let n_chars = value_str.chars().count();

    value_str
        .chars()
        .enumerate()
        .flat_map(|(i, c)| {
            if i != 0 && (n_chars - i).is_multiple_of(3) {
                Some(',')
            } else {
                None
            }
            .into_iter()
            .chain(std::iter::once(c))
        })
        .collect::<String>()
}

pub fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    format!("{hours:02}:{minutes:02}:{seconds:02}")
}

pub fn format_datetime(dt: DateTime<Utc>) -> String {
    Date::new(&JsValue::from_str(&dt.to_rfc3339()))
        .to_locale_string("default", &JsValue::UNDEFINED)
        .as_string()
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comma_format() {
        assert_eq!(comma_format(0.0), "0");
        assert_eq!(comma_format(100.0), "100");
        assert_eq!(comma_format(1000.0), "1,000");
        assert_eq!(comma_format(10000.0), "10,000");
        assert_eq!(comma_format(999999.0), "999,999");
        assert_eq!(comma_format(1000000.0), "1,000,000");
    }

    #[test]
    fn test_number_format() {
        assert_eq!(format_number(0.0), "0");
        assert_eq!(format_number(100.0), "100");
        assert_eq!(format_number(1000.0), "1,000");
        assert_eq!(format_number(10000.0), "10,000");
        assert_eq!(format_number(999999.0), "999,999");
        assert_eq!(format_number(1000000.0), "1.00e6");
    }
}
