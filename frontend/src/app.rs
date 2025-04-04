use leptoaster::*;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use url::Url;

use crate::components::pages;

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

    view! {
        <Toaster stacked={true} />
        <Router base=base_uri>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/") view=pages::MainMenu/>
                <Route path=path!("/connect") view=pages::Connect/>
                <Route path=path!("/game") view=pages::Game/>
            </Routes>
        </Router>
    }
}
