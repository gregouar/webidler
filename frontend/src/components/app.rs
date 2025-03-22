use super::game::Game;
use super::main_menu::MainMenu;
use leptos::prelude::*;
use leptos_router::components::{Route, Router, Routes};
use leptos_router::path;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/") view=MainMenu/>
                <Route path=path!("/game") view=Game/>
            </Routes>
        </Router>
    }
}
