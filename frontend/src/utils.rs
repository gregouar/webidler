use leptos::web_sys;

pub fn now() -> f64 {
    web_sys::window().unwrap().performance().unwrap().now()
}
