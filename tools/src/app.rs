use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};
use leptos_toaster::*;

use crate::pages;

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Toaster position=ToasterPosition::BottomCenter></Toaster>
        <Router>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/") view=pages::HomePage />
                <Route path=path!("/passives") view=pages::PassivesPage />
            </Routes>
        </Router>
    }
}
