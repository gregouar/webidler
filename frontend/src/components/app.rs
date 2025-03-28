use std::borrow::Cow;

use super::game::Game;
use super::main_menu::MainMenu;
use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    let base_uri: Cow<'static, str> = document()
        .base_uri()
        .unwrap_or(Some("/".to_string()))
        .unwrap_or("/".to_string())
        .into();

    view! {
        <Router base=base_uri>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/") view=MainMenu/>
                <Route path=path!("/game") view=Game/>
            </Routes>
        </Router>
    }
}
