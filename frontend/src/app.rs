use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;
use leptos_toaster::*;
use url::Url;

use crate::components::pages;
use crate::components::rest::RestContext;

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

    provide_context(RestContext::new(
        option_env!("BACKEND_URL").unwrap_or("http://localhost:4200"),
    ));

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
