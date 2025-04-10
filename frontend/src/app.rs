use leptoaster::*;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use url::Url;

use crate::components::pages;

// TODO: localization https://crates.io/crates/fluent-templates

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let base_uri = document()
        .base_uri()
        .ok()
        .flatten()
        .and_then(|base| Url::parse(&base).ok())
        .map(|url| url.path().to_string())
        .unwrap_or_else(|| "/".to_string());

    provide_toaster();

    view! {
        <Toaster stacked={true}
            style:--leptoaster-info-background-color="oklch(21% 0.006 285.885)"
            style:--leptoaster-info-border-color="oklch(14.1% 0.005 285.823)"
            style:--leptoaster-info-text-color="white"
        />
        <Router base=base_uri>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/") view=pages::MainMenu/>
                <Route path=path!("/game") view=pages::Game/>
                <Route path=path!("/uimockup") view=pages::UIMockUp/>
                <Route path=path!("/connect") view=pages::WsTest/>
            </Routes>
        </Router>
    }
}
