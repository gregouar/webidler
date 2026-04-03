use frontend::components::{
    accessibility::provide_accessibility_context,
    data_context::provide_data_context,
    events::provide_events_context,
    settings::provide_settings_context,
    ui::{
        confirm::{ConfirmationModal, provide_confirm_context},
        tooltip::DynamicTooltip,
    },
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
    provide_data_context();

    let confirm_state = provide_confirm_context();

    view! {
        <Toaster position=ToasterPosition::BottomCenter></Toaster>
        <ConfirmationModal state=confirm_state />
        <DynamicTooltip />
        <Router>
            <Routes fallback=|| "Page not found.">
                <Route path=path!("/") view=pages::HomePage />
                <Route path=path!("/passives") view=pages::PassivesPage />
                <Route path=path!("/ui_tests") view=pages::UiTestsPage />
            </Routes>
        </Router>
    }
}
