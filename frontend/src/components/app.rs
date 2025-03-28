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

    // let base_uri = move || {
    //     window()
    //         .document()
    //         .map(|doc| doc.base_uri().unwrap_or_default())
    //         .unwrap_or_else(|| Some("/".to_string())) // Default to root
    // };

    // let base_uri = move || document().base_uri().unwrap().unwrap_or("/".to_string());

    let base_uri: Cow<'static, str> = document()
        .base_uri()
        .unwrap_or(Some("/".to_string()))
        .unwrap_or("/".to_string())
        .into();

    println!("{}", base_uri);

    view! {
        <Router base="/">
            <Routes fallback=|| base_uri>
                <Route path=path!("/") view=MainMenu/>
                <Route path=path!("/game") view=Game/>
            </Routes>
        </Router>
    }
}
