use chrono::{DateTime, Utc};
use std::time::Duration;

use leptos::{prelude::*, wasm_bindgen::JsValue, web_sys::js_sys::Date};

use crate::components::settings::SettingsContext;

#[component]
pub fn Number(value: Signal<f64>) -> impl IntoView {
    let settings_context: SettingsContext = expect_context();
    view! {
        {move || {
            format_number_without_context(
                value.get(),
                settings_context.read_settings().scientific_notation,
            )
        }}
    }
}

pub fn format_number(value: f64) -> String {
    let settings_context: SettingsContext = expect_context();
    format_number_without_context(
        value,
        settings_context
            .read_settings_untracked()
            .scientific_notation,
    )
}

pub fn format_number_without_context(value: f64, scientific_notation: bool) -> String {
    if value.is_nan() || value.is_infinite() {
        return value.to_string();
    }

    if value < 0.0 {
        return format!(
            "-{}",
            format_number_without_context(-value, scientific_notation)
        );
    }

    if value < 1_000.0 {
        return comma_format(value);
    }

    if scientific_notation {
        format_scientific_number(value)
    } else {
        format_alphabetic_number(value)
    }
}

fn format_alphabetic_number(value: f64) -> String {
    const SUFFIXES: [&str; 34] = [
        "", "K", "M", "B", "t", "q", "Q", "s", "S", "o", "n", "d", "U", "D", "T", "Qt", "Qd", "Sd",
        "St", "O", "N", "v", "c", "Dvg", "Tvg", "Qav", "Qvg", "Svg", "Spv", "Ovg", "Nvg", "Tg",
        "Utg", "Dtg",
    ];

    let index = (value.log10() / 3.0).floor() as usize;
    if let Some(suffix) = SUFFIXES.get(index) {
        let scaled = value / 1000_f64.powi(index as i32);

        if scaled >= 100.0 {
            format!("{:.0}{}", scaled, suffix)
        } else if scaled >= 10.0 {
            format!("{:.1}{}", scaled, suffix)
        } else {
            format!("{:.2}{}", scaled, suffix)
        }
    } else {
        format_scientific_number(value)
    }
}

fn format_scientific_number(value: f64) -> String {
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

pub fn format_duration(duration: Duration, show_seconds: bool) -> String {
    let mut secs = duration.as_secs();
    if !show_seconds {
        secs = secs.div_ceil(60) * 60;
    }
    let hours = secs / 3600;
    let minutes = (secs % 3600) / 60;
    let seconds = secs % 60;
    if show_seconds {
        format!("{hours}:{minutes:02}:{seconds:02}")
    } else {
        format!("{hours}:{minutes:02}")
    }
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
        assert_eq!(format_number_without_context(0.0, true), "0");
        assert_eq!(format_number_without_context(100.0, true), "100");
        assert_eq!(format_number_without_context(1000.0, true), "1,000");
        assert_eq!(format_number_without_context(10000.0, true), "10,000");
        assert_eq!(format_number_without_context(999999.0, true), "999,999");
        assert_eq!(format_number_without_context(1000000.0, true), "1.00e6");
    }
}
