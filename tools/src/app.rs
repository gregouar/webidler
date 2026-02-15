use frontend::components::{
    accessibility::provide_accessibility_context,
    events::provide_events_context,
    settings::provide_settings_context,
    ui::confirm::{ConfirmationModal, provide_confirm_context},
};
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

    provide_accessibility_context();
    provide_settings_context();
    provide_events_context();

    let confirm_state = provide_confirm_context();

    view! {
        <Toaster position=ToasterPosition::BottomCenter></Toaster>
        <ConfirmationModal state=confirm_state />
        <Router>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/") view=pages::HomePage />
                <Route path=path!("/passives") view=pages::PassivesPage />
            </Routes>
        </Router>
    }
}
