use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use leptos_toaster::*;
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

    view! {
        <Toaster position=ToasterPosition::BottomCenter></Toaster>
        <Router base=base_uri>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/") view=pages::MainMenuPage />
                <Route path=path!("/game") view=pages::GamePage />
                <Route path=path!("/local_game") view=pages::LocalGamePage />
                <Route path=path!("/leaderboard") view=pages::LeaderboardPage />
            </Routes>
        </Router>
    }
}
